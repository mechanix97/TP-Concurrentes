use std::fs;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};


use actix::{Actor, System};
pub use crate::logger::*;

pub use crate::commons::{deserialize_transaction, DistMsg, ExternalMsg};
pub use crate::transaction_writer::*;
pub use crate::connection::*;
pub use crate::actor_transmiter::*;
pub use crate::stats::*;

pub fn exec(
    id: u32,
    connections: Arc<Mutex<Vec<Connection>>>,
    logger: Logger,
    mut commiter: TransactionWriter, 
    mut rollbacker: TransactionWriter,
    running: Arc<Mutex<bool>>
) {
    for c in &mut *connections.lock().unwrap() {
        c.write(DistMsg::NewLeader { id: id });
    }
    
    //sleep(time::Duration::from_secs(5));
      
    let buf_reader = continue_transactions(commiter.clone(), rollbacker.clone());

    logger.log("start processing".to_string());
    
    println!("Empiezo a procesar transacciones");
    let mut bank_connection = TcpStream::connect("127.0.0.1:7878").unwrap();
    let mut airline_connection = TcpStream::connect("127.0.0.1:7879").unwrap();
    let mut hotel_connection = TcpStream::connect("127.0.0.1:7880").unwrap();

    let mut stats = Stats::new();

    for line in buf_reader.lines() {
        if !*running.lock().unwrap(){
            break;
        }
        
        stats.start_transaction();

        let sys = actix::System::new();
        
        let transaction_line = line.unwrap();
        let transaction = deserialize_transaction(transaction_line.clone()).unwrap();        
        let transaction_id = transaction.get_id();

        logger.log(format!("processing transction: {}", transaction_line)); 
        
        let bc = bank_connection.try_clone().unwrap();
        let ac = airline_connection.try_clone().unwrap();
        let hc = hotel_connection.try_clone().unwrap();
       
        let ret = sys.block_on( async { 
            let addr_bank = Transmiter::new(bc).start();
            let addr_airline = Transmiter::new(ac).start();
            let addr_hotel = Transmiter::new(hc).start(); 

            addr_bank.send(PrepareTransaction { transaction: transaction }).await.unwrap();
            addr_airline.send(PrepareTransaction { transaction: transaction }).await.unwrap();
            addr_hotel.send(PrepareTransaction { transaction: transaction }).await.unwrap();

            let res1 = addr_bank.send(PrepareResponse { id:  transaction_id}).await.unwrap();
            let res2 = addr_airline.send(PrepareResponse { id:  transaction_id}).await.unwrap();
            let res3 = addr_hotel.send(PrepareResponse { id:  transaction_id}).await.unwrap();

            res1 && res2 && res3       
        });
        
        System::current().stop();
        sys.run().unwrap();

        if ret {//COMMIT
            logger.log(format!("transction processed OK: {}", transaction_line)); 

            bank_connection.write(ExternalMsg::Commit{id: transaction_id}.to_string().as_bytes()).unwrap();
            airline_connection.write(ExternalMsg::Commit{id: transaction_id}.to_string().as_bytes()).unwrap();
            hotel_connection.write(ExternalMsg::Commit{id: transaction_id}.to_string().as_bytes()).unwrap();
            
            commiter.log(transaction_line.clone());
            let mut cc = connections.lock().unwrap();
            for c in &mut*cc {
                c.write(DistMsg::Commit{transaction: transaction_line.to_string()});
            }  
        } else {//ROLBACK
            logger.log(format!("transction not procesed: {}", transaction_line)); 

            bank_connection.write(ExternalMsg::Rollback{id: transaction_id}.to_string().as_bytes()).unwrap();
            airline_connection.write(ExternalMsg::Rollback{id: transaction_id}.to_string().as_bytes()).unwrap();
            hotel_connection.write(ExternalMsg::Rollback{id: transaction_id}.to_string().as_bytes()).unwrap();

            rollbacker.log(transaction_line.clone());
            let mut cc = connections.lock().unwrap();
            for c in &mut*cc {
                c.write(DistMsg::Rollback{transaction: transaction_line.to_string()});
            }
        }
        stats.end_transaction(ret);
    }
    stats.stop();

    bank_connection.write(ExternalMsg::Stop{stop: true}.to_string().as_bytes()).unwrap();
    airline_connection.write(ExternalMsg::Stop{stop: true}.to_string().as_bytes()).unwrap();
    hotel_connection.write(ExternalMsg::Stop{stop: true}.to_string().as_bytes()).unwrap();


    logger.log("end procesing".to_string());
}



fn continue_transactions(commiter: TransactionWriter, rollbacker: TransactionWriter) -> io::BufReader<fs::File> {
    let filename = r"transactions/input.txt";
    let file = fs::File::open(filename).expect("Error: file not found!");
    let mut ll = String::new() ;
    let reader = io::BufReader::new(file);
    if commiter.last_line().is_none() && rollbacker.last_line().is_none() {
        return reader;
    } else if !commiter.last_line().is_none() && !rollbacker.last_line().is_none(){
        let lc = commiter.last_line().unwrap();
        let lr = rollbacker.last_line().unwrap();
        let tc = deserialize_transaction(lc.clone()).unwrap();
        let tr = deserialize_transaction(lr.clone()).unwrap();
        if tc.get_id() > tr.get_id(){
            ll = lc;
        } else {
            ll = lr;
        }
    } else if commiter.last_line().is_none() {
        ll = rollbacker.last_line().unwrap();
    } else if rollbacker.last_line().is_none() {
        ll = commiter.last_line().unwrap();
    }
    ll = ll.trim().to_string();

    println!("Ultima transaccion: {}", ll);

    let mut count = 0;
    
    for line in reader.lines(){       
        let l = line.unwrap();
        if ll.eq(&l){
            break
        }
        count = count + l.chars().count() + 1;
    }

    let offset_file = fs::File::open(filename).unwrap();
    let mut offset_reader = io::BufReader::new(offset_file);
    offset_reader.seek_relative(count.try_into().unwrap()).unwrap(); 
    offset_reader
}


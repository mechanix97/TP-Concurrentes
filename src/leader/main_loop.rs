use std::fs;
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use std::thread::sleep;
use core::time;

use actix::{Actor, System, Arbiter};
use async_std::task::block_on;
use futures::{join, Future};

pub use crate::logger::*;

pub use crate::commons::{deserialize_transaction, DistMsg};
pub use crate::transaction_writer::*;
pub use crate::connection::*;
pub use crate::actor_transmiter::*;

pub fn exec(
    id: u32,
    connections: Arc<Mutex<Vec<Connection>>>,
    logger: Logger,
    commiter: TransactionWriter, 
    rollbacker: TransactionWriter,
    running: Arc<Mutex<bool>>
) {
    for c in &mut *connections.lock().unwrap() {
        c.write(DistMsg::NewLeader { id: id });
    }
    
    println!("SOY LIDER");
    //sleep(time::Duration::from_secs(5));
      
    let buf_reader = continue_transactions(commiter.clone(), rollbacker.clone());

    logger.log("start processing".to_string());
    
    println!("EMPIEZP");
    let bank_connection = TcpStream::connect("127.0.0.1:7878").unwrap();
    let airline_connection = TcpStream::connect("127.0.0.1:7879").unwrap();
    let hotel_connection = TcpStream::connect("127.0.0.1:7880").unwrap();

    for line in buf_reader.lines() {
        if !*running.lock().unwrap(){
            break;
        }
        
        let sys = actix::System::new();

        let l = line.unwrap();
        //println!("line {}", l);
        //logger.log(format!("processing transction: {}", l));         
        let transaction = deserialize_transaction(l.clone()).unwrap();        
        
        let bc = bank_connection.try_clone().unwrap();
        let ac = airline_connection.try_clone().unwrap();
        let hc = hotel_connection.try_clone().unwrap();
       
        let ret = sys.block_on( async { 
            let addr_bank = Transmiter::new(bc).start();
            let addr_airline = Transmiter::new(ac).start();
            let addr_hotel = Transmiter::new(hc).start();

            let tid= transaction.get_id();

            addr_bank.send(PrepareTransaction { transaction: transaction }).await.unwrap();
            addr_airline.send(PrepareTransaction { transaction: transaction }).await.unwrap();
            addr_hotel.send(PrepareTransaction { transaction: transaction }).await.unwrap();

            let res1 = addr_bank.send(PrepareResponse { id:  tid}).await.unwrap();
            let res2 = addr_airline.send(PrepareResponse { id:  tid}).await.unwrap();
            let res3 = addr_hotel.send(PrepareResponse { id:  tid}).await.unwrap();

            match res1{
                true => (),
                false => (),
            }

        });
        
        System::current().stop();
        //sys.run().unwrap();


        let mut cc = connections.lock().unwrap();
        for c in &mut*cc {
            c.write(DistMsg::Commit{transaction: l.to_string()});
        }
        
    }
    println!("TERMINE");
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


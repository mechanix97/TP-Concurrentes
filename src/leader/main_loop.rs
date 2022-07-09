use core::time;
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::sleep;

pub use crate::logger::*;

pub use crate::commons::{deserialize_transaction, DistMsg};
pub use crate::transaction_writer::*;

pub fn exec(
    id: u32,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    logger: Logger,
    commiter: TransactionWriter, 
    rollbacker: TransactionWriter
) {
    for c in &mut *connections.lock().unwrap() {
        c.0.write(&DistMsg::NewLeader { id: id }.to_string().as_bytes())
            .unwrap();
    }
    println!("SOY LIDER");

    sleep(time::Duration::from_secs(5));
    let _sys = actix::System::new();


    let buf_reader = continue_transactions(commiter.clone(), rollbacker.clone());

    logger.log("start processing".to_string());

    for c in &mut*connections.lock().unwrap() {
       println!("CONEXION ID: {}, {}:{}", c.1, c.2, c.3);
    }


    for line in buf_reader.lines() {
        let l = line.unwrap();
        //logger.log(format!("processing transction: {}", l));
        let transaction = deserialize_transaction(l.clone()).unwrap();

        let mut cc = connections.lock().unwrap();
        for c in &mut*cc {
            c.0.write(&DistMsg::Commit{transaction: transaction.to_string()}.to_string()
                    .as_bytes(),
            ).unwrap();
        }
        
    }
    println!("TERMINE");
    /*   sys.block_on(async {

        let addr_bank = BankActor { bank_connection: TcpStream::connect("127.0.0.1:7878").unwrap() }.start();
        let addr_hotel = HotelActor { hotel_connection: TcpStream::connect("127.0.0.1:7879").unwrap() }.start();
        let addr_airline = AirlineActor { airline_connection: TcpStream::connect("127.0.0.1:7880").unwrap() }.start();

        for line in buf_reader.lines() {
            let transaction = line.unwrap();

            let separated_line: Vec<&str> = transaction.split(',').collect();

            let transaction_id = separated_line[0].trim().to_string().parse::<i32>().unwrap();
            let bank_payment = separated_line[1].trim().to_string().parse::<f32>().unwrap();
            let hotel_payment = separated_line[2].trim().to_string().parse::<f32>().unwrap();
            let airline_payment = separated_line[3].trim().to_string().parse::<f32>().unwrap();

            let result_hotel = addr_hotel.send(ReservationPrice(transaction_id, hotel_payment));

            let result_bank = addr_bank.send(PaymentPrice(transaction_id, bank_payment));

            let result_airline = addr_airline.send(FlightPrice(transaction_id, airline_payment));


            let res = join!(result_hotel, result_bank, result_airline ); //Resultado
            let result0 =match res.0 {
                Ok(val) => {match val{
                    Ok(val) => val,
                    Err(_) => false
                }}
                Err(_) => false
            };
            let result1 =match res.1 {
                Ok(val) => {match val{
                    Ok(val) => val,
                    Err(_) => false
                }}
                Err(_) => false
            };
            let result2 =match res.2 {
                Ok(val) => {match val{
                    Ok(val) => val,
                    Err(_) => false
                }}
                Err(_) => false
            };

            if result0 && result1 && result2 {
                for c in &mut*connections.lock().unwrap() {
                    c.0.write(&(serde_json::to_string(&DistMsg::Commit{transaction: transaction.clone()})
                        .unwrap()
                            + "\n")
                            .as_bytes(),
                    )
                    .unwrap();
                }
            } else {
                for c in &mut*connections.lock().unwrap() {
                    c.0.write(&(serde_json::to_string(&DistMsg::Rollback{transaction: transaction.clone()})
                        .unwrap()
                            + "\n")
                            .as_bytes(),
                    )
                    .unwrap();
                }
            }
        }
    });

    System::current().stop();
    sys.run().unwrap();
    */
    logger.log("end procesing".to_string());
}



fn continue_transactions(commiter: TransactionWriter, rollbacker: TransactionWriter) -> io::BufReader<fs::File> {
    let filename = r"transactions/input.txt";
    let file = fs::File::open(filename).expect("Error: file not found!");
    let mut ll = String::new() ;
    let reader = io::BufReader::new(file);
    if commiter.last_line().is_none() && rollbacker.last_line().is_none() {
        println!("SON NONE");
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


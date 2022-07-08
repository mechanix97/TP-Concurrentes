use chrono::Local;
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub use crate::logger::*;

pub use crate::commons::{deserialize_transaction, DistMsg};

pub fn exec(id: u32, connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) {
    for c in &mut *connections.lock().unwrap() {
        c.0.write(&(serde_json::to_string(&DistMsg::NewLeader { id: id }).unwrap() + "\n").as_bytes())
            .unwrap();
    }
    println!("SOY LIDER");
    let logger = Logger::new(format!("log/replic_{}.txt", id));
    //sleep(time::Duration::from_secs(10));
    let _sys = actix::System::new();

    logger.log(format!(
        "{}: Start processing",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    let filename = r"input.txt";
    let file = fs::File::open(filename).expect("Error: file not found!");
    let buf_reader = io::BufReader::new(file);

    for line in buf_reader.lines() {
        let _transaction = deserialize_transaction(line.unwrap()).unwrap();
        //println!("{:?}", transaction);
    }

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
    logger.log(format!(
        "{}: end procesing",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
}

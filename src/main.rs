use std::future::Future;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use actix::{Actor, ResponseActFuture};
use actix::prelude::*;
use futures::join;
mod hotel;
mod bank;
mod airline;

use actix::spawn;
use chrono::Local;
pub use hotel::HotelActor;
pub use hotel::ReservationPrice;

pub use bank::BankActor;
pub use bank::PaymentPrice;

pub use airline::AirlineActor;
pub use airline::FlightPrice;

pub mod commons;


fn main() {
    let sys = actix::System::new();

    println!("{}:INICIO", Local::now().format("%Y-%m-%d %H:%M:%S"));
    sys.block_on(async {
        let filename = r"transactions.txt";
        let file = fs::File::open(filename).expect("Error: file not found!");
        let  buf_reader = BufReader::new(file);
        
        let addr_bank = BankActor.start();
        let addr_hotel = HotelActor.start();
        let addr_airline = AirlineActor.start();

        for line in buf_reader.lines() {
            let line_str: String = line.unwrap();
            let separated_line: Vec<&str> = line_str.split(',').collect();
    
            let transaction_id = separated_line[0].trim().to_string().parse::<i32>().unwrap();
            let bank_payment = separated_line[1].trim().to_string().parse::<f32>().unwrap();
            let hotel_payment = separated_line[2].trim().to_string().parse::<f32>().unwrap();
            let airline_payment = separated_line[3].trim().to_string().parse::<f32>().unwrap();

            let result_hotel = addr_hotel.send(ReservationPrice(transaction_id, hotel_payment));

            let result_bank = addr_bank.send(PaymentPrice(transaction_id, bank_payment));

            let result_airline = addr_airline.send(FlightPrice(transaction_id, airline_payment)); 

            
            let res = join!(result_hotel, result_bank, result_airline ); //Resultado
            
            match res.0 {
                Ok(_) => println!("HOTEL TERMINO OK"),
                Err(_) => println!("HOTEL ERROR")
            };
            match res.1 {
                Ok(_) => println!("BANCO TERMINO OK"),
                Err(_) => println!("BANCO ERROR")
            };
            match res.2 {
                Ok(_) => println!("AEROLINEA TERMINO OK"),
                Err(_) => println!("AEROLINEA ERROR")
            };
            println!("{}:Termine transaccion", Local::now().format("%Y-%m-%d %H:%M:%S"));


        }        
    });
    System::current().stop();
    sys.run().unwrap();
    println!("{}:FIN", Local::now().format("%Y-%m-%d %H:%M:%S"));

}



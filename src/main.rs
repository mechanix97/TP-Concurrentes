use std::future::Future;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs;
use std::rc::Rc;
use std::sync::Arc;
use actix::Actor;
use actix::prelude::*;

mod hotel;
mod bank;
mod airline;

use actix::spawn;
pub use hotel::HotelActor;
pub use hotel::ReservationPrice;

pub use bank::BankActor;
pub use bank::PaymentPrice;

pub use airline::AirlineActor;
pub use airline::FlightPrice;

pub mod commons;


fn main() {

    let filename = r"transactions.txt";
    let file = fs::File::open(filename).expect("Error: file not found!");
    let  buf_reader = BufReader::new(file);

    let addr_bank = Arc::new(BankActor.start());
    let addr_hotel = Arc::new(HotelActor.start());
    let addr_airline = Arc::new(AirlineActor.start());

    for line in buf_reader.lines() {
        let sys = System::new();

        let line_str: String = line.unwrap();
        let separated_line: Vec<&str> = line_str.split(',').collect();

        let transaction_id = separated_line[0].trim().to_string().parse::<i32>().unwrap();
        let bank_payment = separated_line[1].trim().to_string().parse::<f32>().unwrap();
        let hotel_payment = separated_line[2].trim().to_string().parse::<f32>().unwrap();
        let airline_payment = separated_line[3].trim().to_string().parse::<f32>().unwrap();

        let execution = async {
            alglobo(transaction_id, bank_payment, hotel_payment, airline_payment, addr_airline.clone(), addr_bank.clone(), addr_hotel.clone()).await;
        };

        sys.block_on(execution);
        
        System::current().stop();

        sys.run();
        

        
        
    }
}

async fn alglobo(transaction_id: i32, bank_payment: f32, hotel_payment: f32, airline_payment: f32, addr_airline: Arc<Addr<AirlineActor>>, addr_bank: Arc<Addr<BankActor>>, addr_hotel: Arc<Addr<HotelActor>>) {

    let result_hotel = addr_hotel.send(ReservationPrice(transaction_id, hotel_payment));

    let result_bank = addr_bank.send(PaymentPrice(transaction_id, bank_payment));

    let result_airline = addr_airline.send(FlightPrice(transaction_id, airline_payment)); 

    match result_hotel.await {
        Ok(_) => println!("Got result Ok from Hotel"),
        Err(err) => println!("Got error from Hotel: {}", err),
    }

    match result_bank.await {
        Ok(_) => println!("Got result Ok from Bank"),
        Err(err) => println!("Got error from Bank: {}", err),
    }

    match result_airline.await {
        Ok(_) => println!("Got result Ok from Airline"),
        Err(err) => println!("Got error from Airline: {}", err),
    }
}

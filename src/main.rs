use std::io::prelude::*;
use std::io::BufReader;
use std::fs;
use actix::Actor;
mod hotel;
mod bank;
mod airline;

pub use hotel::HotelActor;
pub use hotel::ReservationPrice;

pub use bank::BankActor;
pub use bank::PaymentPrice;

pub use airline::AirlineActor;
pub use airline::FlightPrice;

pub mod commons;
//mod Payment;

#[actix_rt::main]
async fn main() {

    let filename = r"transactions.txt";
    let file = fs::File::open(filename).expect("Error: file not found!");
    let  buf_reader = BufReader::new(file);

    for line in buf_reader.lines() {
        let line_str: String = line.unwrap();
        let separated_line: Vec<&str> = line_str.split(',').collect();

        let transaction_id = separated_line[0].trim().to_string().parse::<i32>().unwrap();
        let bank_payment = separated_line[1].trim().to_string().parse::<f32>().unwrap();
        let hotel_payment = separated_line[2].trim().to_string().parse::<f32>().unwrap();
        let airline_payment = separated_line[3].trim().to_string().parse::<f32>().unwrap();

        alglobo(transaction_id, bank_payment, hotel_payment, airline_payment).await;
    }
}

async fn alglobo(transaction_id: i32, bank_payment: f32, hotel_payment: f32, airline_payment: f32) {

    let addr_bank = BankActor.start();
    let addr_hotel = HotelActor.start();
    let addr_airline = AirlineActor.start();

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

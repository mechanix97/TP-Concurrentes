use actix::prelude::*;
use core::time;
use std::{net::TcpStream, io::Write, usize, thread::sleep};

use crate::commons;

#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct FlightPrice(pub i32, pub f32);

impl Handler<FlightPrice> for AirlineActor {
    // type Result = ResponseActFuture<Self, Result<bool, std::io::Error>>;
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: FlightPrice, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(async move {
            println!("Received this money: {} from transaction with id {} to pay to the Airline", msg.1, msg.0);

            // let mut airline = TcpStream::connect("127.0.0.1:7880").unwrap();

            // sleep(time::Duration::from_millis(10));
        
            let msg = commons::Payment{id: msg.0, amount: msg.1};

            // airline.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

            Ok(())
        })
    }
}
pub struct AirlineActor;

impl Actor for AirlineActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Airline is alive!");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Actor Airline is stopped");
    }
}
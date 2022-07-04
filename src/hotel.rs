use actix::prelude::*;
use core::time;
use std::{net::TcpStream, io::Write, thread::sleep};

use crate::commons;

#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct ReservationPrice(pub i32, pub f32);

impl Handler<ReservationPrice> for HotelActor {
    // type Result = ResponseActFuture<Self, Result<bool, std::io::Error>>;
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: ReservationPrice, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(async move {
            println!("Received this money: {} from transaction with id {} to pay to the Hotel", msg.1, msg.0);

            // let mut hotel = TcpStream::connect("127.0.0.1:7881").unwrap();

            // sleep(time::Duration::from_millis(1000));
        
            let msg = commons::Payment{id: msg.0, amount: msg.1};

            // hotel.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

            Ok(())
        })
    }
}
pub struct HotelActor;

impl Actor for HotelActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Hotel is alive!");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Actor Hotel is stopped");
    }
}
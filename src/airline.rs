use actix::prelude::*;
use core::time;
use std::{net::TcpStream, io::Write, usize, thread::sleep};

use crate::commons::commons::{self};

pub struct FlightPrice(pub i32, pub f32);

impl Message for FlightPrice {
    type Result = ();
}

impl Handler<FlightPrice> for AirlineActor {
    type Result = ();

    fn handle(&mut self, msg: FlightPrice, ctx: &mut Context<Self>) -> Self::Result {
        let future = Box::pin(async move {
            println!("Received this money: {} from transaction with id {} to pay to the Airline", msg.1, msg.0);

            // let mut airline = TcpStream::connect("127.0.0.1:7880").unwrap();

            // sleep(time::Duration::from_millis(10));
        
            let msg = commons::Payment{id: msg.0, amount: msg.1};

            // airline.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

        });
        let actor_future = future.into_actor(self);

        ctx.spawn(actor_future);
    }
}
pub struct AirlineActor;

impl Actor for AirlineActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Airline is alive!");
        ctx.notify(FlightPrice(1, 300.0));
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Airline is stopped");
    }
}
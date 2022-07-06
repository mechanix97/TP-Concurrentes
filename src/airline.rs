use actix::prelude::*;
use chrono::Local;
use core::time;
use std::{net::TcpStream, io::Write, usize, thread::sleep};
use async_std::task;

use crate::payment::Payment;


#[derive(Message)]
#[rtype(result = "Result<bool, ()>")]
pub struct FlightPrice(pub i32, pub f32);

impl Handler<FlightPrice> for AirlineActor {
    type Result = ResponseActFuture<Self, Result<bool, ()>>;

    fn handle(&mut self, msg: FlightPrice, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(
            async {
            
                let msg = Payment{id: msg.0, amount: msg.1};

                self.airline_connection.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

                // task::sleep(time::Duration::from_secs(3)).await;
                true
            }
            .into_actor(self) // converts future to ActorFuture
            .map(|res, _act, _ctx| {
                // Do some computation with actor's state or context
                println!("{}: Mensaje aerolinea", Local::now().format("%Y-%m-%d %H:%M:%S"));
                Ok(res)
            }),
        )
    }
}
pub struct AirlineActor { airline_connection: TcpStream }

impl Actor for AirlineActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Airline is alive!");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Airline is stopped");
    }
}
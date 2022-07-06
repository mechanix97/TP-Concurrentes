use actix::prelude::*;
use chrono::Local;
use core::time;
use std::{net::TcpStream, io::Write, thread::sleep};

use crate::payment::Payment;

#[derive(Message)]
#[rtype(result = "Result<bool, ()>")]
pub struct ReservationPrice(pub i32, pub f32);

impl Handler<ReservationPrice> for HotelActor {
    type Result = ResponseActFuture<Self, Result<bool, ()>>;

    fn handle(&mut self, msg: ReservationPrice, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(
            async {

                let msg = Payment{id: msg.0, amount: msg.1};

                self.hotel_connection.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

                // task::sleep(time::Duration::from_secs(6)).await;
                true
            }
            .into_actor(self) // converts future to ActorFuture
            .map(|res, _act, _ctx| {
                // Do some computation with actor's state or context
                println!("{}: Mensaje Hotel", Local::now().format("%Y-%m-%d %H:%M:%S"));
                Ok(res)
            }),
        )
    }
}
pub struct HotelActor { hotel_connection: TcpStream }

impl Actor for HotelActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Hotel is alive!");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Hotel is stopped");
    }
}

use actix::prelude::*;
use core::time;
use std::{net::TcpStream, io::Write, thread::sleep, pin::Pin, future::Future};

use crate::commons::commons::{self};

pub struct ReservationPrice(pub i32, pub f32);

impl Message for ReservationPrice {
    type Result = ();
}

impl Handler<ReservationPrice> for HotelActor {
    type Result = ();

    fn handle(&mut self, msg: ReservationPrice, ctx: &mut Context<Self>) -> Self::Result {
        let future = Box::pin(async move {

            // let mut hotel = TcpStream::connect("127.0.0.1:7881").unwrap();

            sleep(time::Duration::from_millis(3000));

            println!("Received this money: {} from transaction with id {} to pay to the Hotel", msg.1, msg.0);
        
            let msg = commons::Payment{id: msg.0, amount: msg.1};

            // hotel.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

        });
        let actor_future = future.into_actor(self);

        ctx.wait(actor_future);

    }
}
pub struct HotelActor;

impl Actor for HotelActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Hotel is alive!");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Hotel is stopped");
    }
}

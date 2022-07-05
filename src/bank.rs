use actix::prelude::*;
use core::time;
use std::{net::TcpStream, io::Write, usize, thread::sleep};

use crate::commons::commons::{self};

pub struct PaymentPrice(pub i32, pub f32);

impl Message for PaymentPrice {
    type Result = ();
}

impl Handler<PaymentPrice> for BankActor {
    // type Result = ResponseActFuture<Self, Result<bool, std::io::Error>>;
    type Result = ();

    fn handle(&mut self, msg: PaymentPrice, ctx: &mut Context<Self>) -> Self::Result {
        let future = Box::pin(async move {
            println!("Received this money: {} from transaction with id {} to pay to the Bank", msg.1, msg.0);

            // let mut bank = TcpStream::connect("127.0.0.1:7879").unwrap();
        
            let msg = commons::Payment{id: msg.0, amount: msg.1};

            // sleep(time::Duration::from_millis(1000));

            // bank.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

        });
        let actor_future = future.into_actor(self);

        ctx.wait(actor_future);
    }
}
pub struct BankActor;

impl Actor for BankActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Bank is alive!");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Bank is stopped");
    }
}
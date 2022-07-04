use actix::prelude::*;
use core::time;
use std::{net::TcpStream, io::Write, usize, thread::sleep};

use crate::commons::commons::{self};

#[derive(Message)]
#[rtype(result = "Result<(), std::io::Error>")]
pub struct PaymentPrice(pub i32, pub f32);

impl Handler<PaymentPrice> for BankActor {
    // type Result = ResponseActFuture<Self, Result<bool, std::io::Error>>;
    type Result = ResponseFuture<Result<(), std::io::Error>>;

    fn handle(&mut self, msg: PaymentPrice, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(async move {
            println!("Received this money: {} from transaction with id {} to pay to the Bank", msg.1, msg.0);

            // let mut bank = TcpStream::connect("127.0.0.1:7879").unwrap();
        
            let msg = commons::Payment{id: msg.0, amount: msg.1};

            // sleep(time::Duration::from_millis(1000));

            // bank.write_all(&(serde_json::to_string(&msg).unwrap()+"\n").as_bytes()).unwrap();

            Ok(())
        })
    }
}
pub struct BankActor;

impl Actor for BankActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Bank is alive!");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Actor Bank is stopped");
    }
}
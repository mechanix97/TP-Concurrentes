use actix::prelude::*;
use chrono::Local;
use core::time;
use std::{net::TcpStream, io::Write, usize, thread::sleep, pin::Pin, future::Future};
use async_std::task;
pub struct PaymentPrice(pub i32, pub f32);

impl Message for PaymentPrice {
    type Result = Result<bool, ()>;
}

impl Handler<PaymentPrice> for BankActor {
    type Result = ResponseActFuture<Self, Result<bool, ()>>;

    fn handle(&mut self, msg: PaymentPrice, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(
            async {
                // Some async computation
                task::sleep(time::Duration::from_secs(3)).await;
                true
            }
            .into_actor(self) // converts future to ActorFuture
            .map(|res, _act, _ctx| {
                // Do some computation with actor's state or context
                println!("{}: Mensaje banco", Local::now().format("%Y-%m-%d %H:%M:%S"));
                Ok(res)
            }),
        )
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
use actix::prelude::*;
use std::io::{self, BufRead};
use std::{io::Write, net::TcpStream};

use crate::commons::{deserialize_ext, ExternalResponse, Payment};

#[derive(Message)]
#[rtype(result = "Result<bool, ()>")]
pub struct FlightPrice(pub i32, pub f32);

impl Handler<FlightPrice> for AirlineActor {
    type Result = ResponseActFuture<Self, Result<bool, ()>>;

    fn handle(&mut self, msg: FlightPrice, _ctx: &mut Context<Self>) -> Self::Result {
        let msg = Payment {
            id: msg.0,
            amount: msg.1,
        };

        let mut writer = self.airline_connection.try_clone().unwrap();
        let mut reader = io::BufReader::new(self.airline_connection.try_clone().unwrap());
        Box::pin(
            async move {
                let ret;
                writer
                    .write_all(&(serde_json::to_string(&msg).unwrap() + "\n").as_bytes())
                    .unwrap();
                let mut s = String::new();

                let len = match reader.read_line(&mut s) {
                    Ok(val) => val,
                    Err(_err) => 0,
                };
                if s.is_empty() || len == 0 {
                    ret = false;
                } else {
                    ret = match deserialize_ext(s.trim_end().to_string()) {
                        Ok(m) => match m {
                            ExternalResponse::ACK => true,
                            ExternalResponse::NACK => false,
                        },
                        Err(_) => false,
                    };
                }
                ret
            }
            .into_actor(self) // converts future to ActorFuture
            .map(|res, _act, _ctx| {
                // Do some computation with actor's state or context
                Ok(res)
            }),
        )
    }
}
pub struct AirlineActor {
    pub airline_connection: TcpStream,
}

impl Actor for AirlineActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Airline is alive!");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Airline is stopped");
    }
}

use actix::prelude::*;
use std::io::{self, BufRead};
use std::thread::{JoinHandle, self};
use std::{io::Write, net::TcpStream};

use crate::commons::{deserialize_ext, ExternalMsg, Transaction};

#[derive(Copy, Clone)]
enum TransactionStatus{
    Waiting,
    Nack,
    Ack
}

pub struct Transmiter {
    stream: TcpStream,
    handle: Option<JoinHandle<()>>,
    status: TransactionStatus
}

impl Transmiter{
    pub fn new(stream: TcpStream) -> Transmiter {        
        Transmiter {
            stream: stream,
            handle: None,
            status: TransactionStatus::Waiting
        }
    }
}

#[derive(Message)]
#[rtype(result = "")]
pub struct PrepareTransaction{
    pub transaction: Transaction
}

impl Handler<PrepareTransaction> for Transmiter {
    type Result = ();

    fn handle(&mut self, msg: PrepareTransaction, _ctx: &mut Context<Self>) -> Self::Result {
        let msg = ExternalMsg::Prepare{transaction: msg.transaction};
        self.stream.write_all(msg.to_string().as_bytes()).unwrap();
        
    }
}

#[derive(Message)]
#[rtype(result = "")]
pub struct AckResponseMsg{}

impl Handler<AckResponseMsg> for Transmiter {
    type Result = ();
    fn handle(&mut self, _msg: AckResponseMsg, _ctx: &mut Context<Self>) -> Self::Result {  
        self.status = TransactionStatus::Ack;
    }
}

#[derive(Message)]
#[rtype(result = "")]
pub struct NackResponseMsg{}

impl Handler<NackResponseMsg> for Transmiter {
    type Result = ();
    fn handle(&mut self, _msg: NackResponseMsg, _ctx: &mut Context<Self>) -> Self::Result {  
        self.status = TransactionStatus::Nack;
    }
}

#[derive(Message)]
#[rtype(result = "bool")]
pub struct PrepareResponse{
    pub id: u32
}

impl Handler<PrepareResponse> for Transmiter {
    type Result = ResponseFuture<bool>;

    fn handle(&mut self, msg: PrepareResponse, ctx: &mut Context<Self>) -> Self::Result {       
        let add = ctx.address(); 
        match self.status {
            TransactionStatus::Nack => Box::pin(async move{false}),
            TransactionStatus::Ack => Box::pin(async move{true}),
            TransactionStatus::Waiting => {
                Box::pin(async move {
                    actix_rt::task::yield_now().await;
                    add.send(msg).await.unwrap()
                })
            }
        }
    }   
}



impl Actor for Transmiter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut reader = io::BufReader::new(self.stream.try_clone().unwrap());
        let add = ctx.address();
        let t = thread::spawn(move ||loop {
            let mut s = String::new();

            let len = match reader.read_line(&mut s) {
                Ok(val) => val,
                Err(_err) => 0,
            };
            if s.is_empty() || len == 0 {
                continue;
            } else {
            match deserialize_ext(s.trim_end().to_string()) {
                    Ok(m) => match m {
                        ExternalMsg::ACK{id: _} => {
                            add.try_send(AckResponseMsg{}).unwrap();
                        },
                        ExternalMsg::NACK{id: _ } =>  {    
                            add.try_send(NackResponseMsg{}).unwrap();
                        },
                        ExternalMsg::Stop{stop} => {if stop {break}},
                        _ => ()
                    },
                    Err(_) => println!("ERR"),
                };
            }
        });
        self.handle = Some(t);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.stream.write_all(ExternalMsg::Stop{stop: false}.to_string().as_bytes())
        .unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}
   
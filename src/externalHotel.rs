use std::io::{self, BufRead, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use rand::distributions::{Uniform};
use rand::prelude::Distribution;

pub use crate::commons::commons::{*};
pub use crate::lib::lib::{*};

mod commons;
mod lib;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7880").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = io::BufReader::new(&mut stream);
    let mut rng = rand::thread_rng();
    let distr: Uniform<i32> = Uniform::from(1..10);

    loop {
        let mut s = String::new();

        let value = distr.sample(&mut rng);

        let len = match reader.read_line(&mut s){
            Ok(val) => val,
            Err(_) => break,
        };
        if value < 4 || s.is_empty() || len == 0 {
            match writer.write(&(serde_json::to_string(&ExternalResponse::NACK) 
            .unwrap()
            + "\n")
                .as_bytes(),
            ){
                Ok(_) => {}
                Err(_) => break
            }
            
        } else {
            match deserialize_pay(s.to_string()) {
                Ok(_) => {
                    match writer.write(&(serde_json::to_string(&ExternalResponse::ACK) 
                    .unwrap()
                        + "\n")
                        .as_bytes(),
                    ){
                        Ok(_) => {}
                        Err(_) => break
                    }
                }
                Err(_) => {
                    match writer.write(&(serde_json::to_string(&ExternalResponse::NACK) 
                    .unwrap()
                        + "\n")
                        .as_bytes(),
                    ){
                        Ok(_) => {}
                        Err(_) => break
                    }
                }
            };
        }
    }    
}

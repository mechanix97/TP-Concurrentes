use rand::distributions::Uniform;
use rand::prelude::Distribution;
use core::time;
use std::io::{self, BufRead, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread::sleep;

mod commons;
mod lib;

use crate::commons::{deserialize_ext, ExternalMsg};
use crate::lib::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
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
  
    // let mut rng = rand::thread_rng();
    // let distr: Uniform<i32> = Uniform::from(1..10);

    loop {
        let mut s = String::new();
        let len = match reader.read_line(&mut s) {
            Ok(val) => val,
            Err(_) => continue,
        };
        if s.is_empty() || len == 0 {
            continue;
        }

        match deserialize_ext( s ) {
            Ok(v) => {
                match v{
                    ExternalMsg::Prepare {transaction: t} => {
                        writer.write(ExternalMsg::ACK{id: t.get_id()}.to_string().as_bytes()).unwrap();
                    },
                    ExternalMsg::ACK{id} => {

                    }
                    ExternalMsg::NACK {id}=> {

                    }
                    ExternalMsg::Stop => {
                        writer.write(ExternalMsg::Stop.to_string().as_bytes()).unwrap();
                    }
                }            
            }
            Err(_) => continue,
        }
    }
}

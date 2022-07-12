use rand::Rng;
use std::io::{self, BufRead, Write};
use std::net::TcpListener;
use std::net::TcpStream;

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
                        let mut rng = rand::thread_rng();
                        let random = rng.gen::<f64>();
                        if random < 0.1 {
                            writer.write(ExternalMsg::NACK{id: t.get_id()}.to_string().as_bytes()).unwrap();
                        } else{
                            writer.write(ExternalMsg::ACK{id: t.get_id()}.to_string().as_bytes()).unwrap();
                        }                       
                    },
                    ExternalMsg::Stop => {
                        writer.write(ExternalMsg::Stop.to_string().as_bytes()).unwrap();
                    },
                    _ => (),
                }            
            }
            Err(_) => continue,
        }
    }
}

use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use rand::distributions::{Distribution, Uniform};

pub use crate::commons::commons::*;
pub use crate::lib::lib::*;
mod commons;
mod lib;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7879").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut reader = io::BufReader::new(&mut stream);
    let mut rng = rand::thread_rng();
    let distr = Uniform::from(1..10);

    loop {
        let mut s = String::new();

        let value = distr.sample(&mut rng);

        let len = match reader.read_line(&mut s) {
            Ok(len) => match mock_response(len, value) {
                    Ok(val) if val > 0 => val,
                    Ok(val) => 0,
                    Err(_err) => 0,
                },
            Err(_err) => 0,        
        };
        
        if s.is_empty() || len == 0 {
            return;
        }
        match deserialize(s.to_string()) {
            Ok(val) => println!("Payment to the Airline was successful for the transaction id {} and the amount {}", id, amount),
            Err(err) => println!("Error in payment to the Airline for transaction id {}: {}", err),
        };
    }
}

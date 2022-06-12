use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use tp::ThreadPool;
mod commons;
mod logger;

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
    loop {
        let mut s = String::new();

        let len = match reader.read_line(&mut s) {
            Ok(val) => val,
            Err(_err) => 0,
        };
        if s.is_empty() || len == 0 {
            return;
        }
        match commons::deserialize(s.to_string()) {
            Ok(val) => match val {
                commons::Msg::Payment { id, amount } => {
                    println!("Payment: {}, {}", id, amount)
                }
                commons::Msg::Reversal { id } => {
                    println!("Reversal: {}", id)
                }
                commons::Msg::Ack => {
                    println!("ACK")
                }
                commons::Msg::Nack => {
                    println!("NACK")
                }
                commons::Msg::Quit => {
                    println!("QUIT")
                }
            },
            Err(err) => println!("ERROR: {}", err),
        };
    }
}

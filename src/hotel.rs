use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use tp::ThreadPool;
mod commons;
mod logger;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7879").unwrap();
    let pool = ThreadPool::new(4);
    let logger = logger::Logger::new("hotel.log".to_string());

    
    for stream in listener.incoming() {
        let l = logger.clone();
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream, l);
        });
    }
}

fn handle_connection(mut stream: TcpStream, logger: logger::Logger) {
    logger.log(format!("New connection: from {}\n", stream.peer_addr().unwrap()));
    let mut reader = io::BufReader::new(&mut stream);
    loop {
        let mut s = String::new();

        let len = match reader.read_line(&mut s) {
            Ok(val) => val,
            Err(_err) => 0,
        };
        if s.is_empty() || len == 0 {
            break;
        }
        match commons::deserialize(s.to_string()) {
            Ok(val) => match val {
                commons::Msg::Payment { id, amount } => {
                    logger.log(format!("Payment from {} with amount {}\n", id, amount))
                }
                commons::Msg::Reversal { id } => {
                    logger.log(format!("Reversal transaction {}\n", id))
                }
                commons::Msg::Ack => {
                    logger.log(format!("ACK\n"))
                }
                commons::Msg::Nack => {
                    logger.log(format!("NACK\n"))
                }
                commons::Msg::Quit => {
                    logger.log(format!("QUIT\n"));
                }
            },
            Err(err) => {
                println!("ERROR: {}", err);
                break;
            },
        };
    }
    logger.log(format!("Connection closed from: {}\n", stream.peer_addr().unwrap()));
}

use std::net::{TcpListener, TcpStream};
use std::io::{self, BufRead};
mod commons;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut reader = io::BufReader::new(&mut stream);
    loop{
        let mut s = String::new();

        let _len = match reader.read_line(&mut s){
            Ok(val) => val,
            Err(_err) => 0
        };
        if s.is_empty() {
            return;
        }
        match commons::deserialize(s.to_string()) {
            Ok(val) => match val {
                commons::Msg::Payment{id, amount} => {println!("Payment: {}, {}",id, amount)},
                commons::Msg::Reversal{id} => {println!("Reversal: {}",id)},
                commons::Msg::Ack => {println!("ACK")},
                commons::Msg::Nack => {println!("NACK")},
                commons::Msg::Quit => {println!("QUIT")}
            },
            Err(err) => println!("ERROR: {}", err)
        };
    }
}

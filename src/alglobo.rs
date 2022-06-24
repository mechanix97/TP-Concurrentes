use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;
use std::fs;
mod commons;
//mod Payment;

fn main() {
    let mut bank = TcpStream::connect("127.0.0.1:7879").unwrap();
    let mut airline = TcpStream::connect("127.0.0.1:7880").unwrap();
    let mut hotel = TcpStream::connect("127.0.0.1:7881").unwrap();

    let filename = r"input.txt";
    let file = File::open(filename).expect("file not found!");
    let  buf_reader = BufReader::new(file);

    for line in buf_reader.lines() {
        println!("{}", line?);
    }

}



use std::env;
use std::io;

mod actor_airline;
mod actor_bank;
mod actor_hotel;
mod alglobo;
mod commons;
mod leader;
mod lib;
mod logger;
mod replic;
mod common;
mod transaction_writer;

pub use crate::alglobo::*;

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut replic;
    if args.len() == 5 {
        replic = Alglobo::new(args[1].parse::<u32>().unwrap(), &args[2], &args[3]);
        replic.start_as_leader();

        read_q();
        replic.join();
    } else if args.len() == 6 {
        replic = Alglobo::new(args[1].parse::<u32>().unwrap(), &args[2], &args[3]);
        replic.start_as_replic(args[4].to_string(), args[5].to_string());

        read_q();

        replic.join();
    } else {
        println!("USAGE: ./alglobo <id> <host> <port> -L <host_leader> <port_leader>");
    }
}

fn read_q() {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if !input.contains("q") {
                read_q()
            }
        }
        Err(error) => println!("error: {}", error),
    }
}

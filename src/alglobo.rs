use std::io;
use std::env;

mod commons;
mod lib;
mod replic;
mod hotel;
mod bank;
mod airline;

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut replic;
    if args.len() == 5 {
        replic = crate::replic::replic::Replic::new(args[1].parse::<u32>().unwrap(), &args[2], &args[3]);
        replic.start_as_leader();

        read_q();

        replic.join();

    } else if args.len() == 6 {
        replic = crate::replic::replic::Replic::new(args[1].parse::<u32>().unwrap(), &args[2], &args[3]);
        replic.start_as_replic(&args[4], &args[5]);

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

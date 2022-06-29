use std::io;

mod commons;
mod lib;
mod replic;

fn main() {
    /* USAGE: ./alglobo <id> <host> <port> -L <host_leader> <port_leader>
        Parametro -L si es leader
    */
    /*if -l {
        leeader.rs <id> <host> <port>
    } else {
        replica <id> <host> <port> <host_leader> <port_leader>
    }*/

    
    let mut replic1 = crate::replic::replic::Replic::new(1, "127.0.0.1", "9000");
    replic1.start_as_leader();


    let mut replic1 = crate::replic::replic::Replic::new(1, "127.0.0.1", "9001");
    replic1.start_as_replic("127.0.0.1", "9000");


    let mut replic2 = crate::replic::replic::Replic::new(2, "127.0.0.1", "9002");
    replic2.start_as_replic("127.0.0.1", "9000");

    let mut replic3 = crate::replic::replic::Replic::new(3, "127.0.0.1", "9003");
    replic3.start_as_replic("127.0.0.1", "9000");


    let mut replic4 = crate::replic::replic::Replic::new(4, "127.0.0.1", "9004");
    replic4.start_as_replic("127.0.0.1", "9000");

    read_q();

    replic1.join();
    replic2.join();
    replic3.join();
    replic4.join();
}

fn read_q() {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            if !input.contains("q") {
                read_q()
            }
        }
        Err(error) => println!("error: {}", error),
    }
}

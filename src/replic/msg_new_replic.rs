use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{atomic::Ordering, Arc, Mutex};

pub use crate::logger::*;

pub fn exec(
    id: u32,
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    leader_alive: Arc<AtomicBool>,
    logger: Logger
) {
    leader_alive.store(true, Ordering::Relaxed);
    let newreplicstream = match TcpStream::connect(format!("{}:{}", hostname, port)){
        Ok(c) => {logger.log(format!("connection established: {}:{}", hostname, port)); c },
        Err(e) => {logger.log(format!("unable to establish connection: {}:{}, ERROR: {}", hostname, port, e)); return;}
    };

    ({
        connections
            .lock()
            .unwrap()
            .push((newreplicstream, id, hostname, port, false));
    });
}

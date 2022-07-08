use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{atomic::Ordering, Arc, Mutex};

pub fn exec(
    id: u32,
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    leader_alive: Arc<AtomicBool>,
) {
    leader_alive.store(true, Ordering::Relaxed);
    let newreplicstream = TcpStream::connect(format!("{}:{}", hostname, port)).unwrap();

    ({
        connections
            .lock()
            .unwrap()
            .push((newreplicstream, id, hostname, port, false));
    });
}

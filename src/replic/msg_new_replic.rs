use std::sync::atomic::AtomicBool;
use std::sync::{atomic::Ordering, Arc, Mutex};

pub use crate::logger::*;
pub use crate::connection::*;

pub fn exec(
    id: u32,
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<Connection>>>,
    leader_alive: Arc<AtomicBool>,
    logger: Logger
) {
    logger.log(format!("new replic: {}, {}:{}",id, hostname, port ));
    leader_alive.store(true, Ordering::Relaxed);
    
    let newconnection = Connection::new(id, hostname, port);

    ({
        connections
            .lock()
            .unwrap()
            .push(newconnection);
    });
}

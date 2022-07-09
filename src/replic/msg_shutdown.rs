use std::sync::{Arc, Mutex};

pub use crate::commons::DistMsg;
pub use crate::connection::*;

pub fn exec(
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<Connection>>>,
) {
    for c in &mut *connections.lock().unwrap() {
        if c.eq(&hostname, &port) {
            c.write(
                DistMsg::Shutdown {
                    hostname: hostname.clone(),
                    port: port.clone(),
                    shutdown: true,
                }
            );
        }
    }
}

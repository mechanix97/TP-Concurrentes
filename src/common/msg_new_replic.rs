use std::sync::{Arc, Mutex};

pub use crate::logger::*;
pub use crate::connection::*;

pub fn exec(
    id: u32,
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<Connection>>>,
    logger: Logger
) {
    logger.log(format!("new replic: {}, {}:{}",id, hostname, port ));
    
    let newconnection = Connection::new(id, hostname, port);

    ({
        connections
            .lock()
            .unwrap()
            .push(newconnection);
    });
}

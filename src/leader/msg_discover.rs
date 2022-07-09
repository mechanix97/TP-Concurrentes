use std::sync::{Arc, Mutex};

pub use crate::commons::DistMsg;
pub use crate::connection::*;

pub fn exec(
    id: u32,
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<Connection>>>,
) {
    //conexion a replica entrante
    let mut newconnection = Connection::new(id, hostname.clone(), port.clone());

    for c in &mut *connections.lock().unwrap() {
        //le informo a las demas replicas de la nueva replica
        c.write(DistMsg::NewReplic {
                    id: id,
                    hostname: hostname.clone(),
                    port: port.clone(),
                }
            );

        //Le informo a la nueva replica de las demas replicas
        newconnection.write(
        DistMsg::NewReplic {
            id: c.get_id(),
            hostname: c.get_hostname(),
            port: c.get_port(),
        });
    }

    //agrego la nueva replica a las conexiones
    ({
        connections
            .lock()
            .unwrap()
            .push(newconnection);
    });
}

use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub use crate::commons::DistMsg;

pub fn exec(
    id: u32,
    hostname: String,
    port: String,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
) {
    //conexion a replica entrante
    let mut newreplicstream = TcpStream::connect(format!("{}:{}", hostname, port)).unwrap();

    for conn in &mut *connections.lock().unwrap() {
        //le informo a las demas replicas de la nueva replica
        conn.0
            .write(
                &(serde_json::to_string(&DistMsg::NewReplic {
                    id: id,
                    hostname: hostname.clone(),
                    port: port.clone(),
                })
                .unwrap()
                    + "\n")
                    .as_bytes(),
            )
            .unwrap();

        //Le informo a la nueva replica de las demas replicas
        newreplicstream
            .write(
                &(serde_json::to_string(&DistMsg::NewReplic {
                    id: conn.1,
                    hostname: conn.2.clone(),
                    port: conn.3.clone(),
                })
                .unwrap()
                    + "\n")
                    .as_bytes(),
            )
            .unwrap();
    }

    //agrego la nueva replica a las conexiones
    ({
        connections
            .lock()
            .unwrap()
            .push((newreplicstream, id, hostname, port, false));
    });
}

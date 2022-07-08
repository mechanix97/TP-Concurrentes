use std::io::Write;
use std::net::TcpStream;
use std::sync::{ Arc, Mutex};

pub use crate::commons::DistMsg;

pub fn exec(hostname: String, port: String, connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>){
    for c in &mut *connections.lock().unwrap(){
        if c.2.eq(&hostname) && c.3.eq(&port){
            c.0.write(
                &(serde_json::to_string(&DistMsg::Shutdown {
                    hostname: hostname.clone(),
                    port: port.clone(),
                    shutdown: true,
                })
                .unwrap()
                    + "\n")
                    .as_bytes(),
            )
            .unwrap();
        }
    }
}
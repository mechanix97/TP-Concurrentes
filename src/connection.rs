use std::net::TcpStream;
pub use crate::commons::DistMsg;
use std::io::Write;

pub struct Connection {
    stream: Option<TcpStream>,
    id: u32,
    hostname: String,
    port: String,
    is_leader: bool,
    is_alive: bool
}

impl Connection{
    pub fn new(id: u32, host: String, port: String) -> Connection {
        match TcpStream::connect(format!(
            "{}:{}",
            host.clone(),
            port.clone()
        )){
            Ok(s) => Connection {
                stream: Some(s),
                id: id,
                hostname: host,
                port: port,
                is_leader: false,
                is_alive: true
            },
            Err(_) => Connection {
                stream: None,
                id: id,
                hostname: host,
                port: port,
                is_leader: false,
                is_alive: false
            }
        }
    }

    pub fn set_as_leader(&mut self){
        self.is_leader = true;
    }

    pub fn write(&mut self, msg: DistMsg){
        if !self.is_alive {
            return;
        }
        match &mut self.stream {
            Some(s) => {
                match s.write(msg.to_string().as_bytes()) {
                    Ok(_) => {}
                    Err(_) => {self.is_alive = false}
                }
            },
            None => (),
        }       
    }

    pub fn eq(&self, host: &String, port: &String) -> bool {
        self.hostname.eq(host) && self.port.eq(port)
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader
    }

    pub fn get_stream(&mut self) -> TcpStream {
        self.stream.as_mut().unwrap().try_clone().unwrap()            
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn set_id(&mut self, id: u32){
        self.id = id;
    }
 

    pub fn get_hostname(&self) -> String {
        self.hostname.clone()
    }

    pub fn get_port(&self) -> String {
        self.port.clone()
    }
}
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub use crate::commons::*;
pub use crate::lib::*;

pub struct Replic {
    id: u32,
    hostname: String,
    port: String,
    is_leader: bool,
    join_handle: Option<JoinHandle<()>>,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String)>>>,
    threadpool: Arc<lib::ThreadPool>,
}

impl Replic {
    pub fn join(self: &mut Self) {
        drop(&self.threadpool);

        match self.join_handle.take() {
            Some(h) => {
                h.join().unwrap();
            }
            None => {}
        }
    }

    pub fn new(id: u32, hostname: &str, port: &str) -> Replic {
        Replic {
            id: id,
            hostname: hostname.to_string(),
            port: port.to_string(),
            is_leader: false,
            join_handle: None,
            connections: Arc::new(Mutex::new(vec![])),
            threadpool: Arc::new(lib::ThreadPool::new(100)),
        }
    }

    pub fn start_as_leader(self: &mut Self) {
        self.is_leader = true;

        let h = self.hostname.clone();
        let p = self.port.clone();
        let connections = self.connections.clone();
        let pool = self.threadpool.clone();
        
        let t = thread::spawn(move || {
            let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();
            for stream in listener.incoming() {
            
                let pool2 = pool.clone();
                let connections_pool = connections.clone();
                pool2.execute(move || {
                    let mut reader = io::BufReader::new(stream.unwrap());
                    loop {
                        let mut s = String::new();

                        let len = match reader.read_line(&mut s) {
                            Ok(val) => val,
                            Err(_err) => 0,
                        };
                        if s.is_empty() || len == 0 {
                            return;
                        }
                        match commons::deserialize_dist(s.to_string()) {
                            Ok(val) => match val {
                                commons::DistMsg::Discover { id, hostname, port } => {
                                    //conexion a replica entrante
                                    let mut newreplicstream =
                                        TcpStream::connect(format!("{}:{}", hostname, port))
                                            .unwrap();

                                    for conn in &mut *&mut *connections_pool.lock().unwrap() {
                                        //le informo a las demas replicas de la nueva replica
                                        conn.0
                                            .write(
                                                &(serde_json::to_string(
                                                    &commons::DistMsg::NewReplic {
                                                        id: id,
                                                        hostname: hostname.clone(),
                                                        port: port.clone(),
                                                    },
                                                )
                                                .unwrap()
                                                    + "\n")
                                                    .as_bytes(),
                                            )
                                            .unwrap();

                                        //Le informo a la nueva replica de las demas replicas
                                        newreplicstream
                                            .write(
                                                &(serde_json::to_string(
                                                    &commons::DistMsg::NewReplic {
                                                        id: conn.1,
                                                        hostname: conn.2.clone(),
                                                        port: conn.3.clone(),
                                                    },
                                                )
                                                .unwrap()
                                                    + "\n")
                                                    .as_bytes(),
                                            )
                                            .unwrap();
                                    }

                                    //agrego la nueva replica a las conexiones
                                    ({
                                        connections_pool.lock().unwrap().push((
                                            newreplicstream,
                                            id,
                                            hostname,
                                            port,
                                        ));
                                    });
                                }
                                commons::DistMsg::NewReplic { id, hostname, port } => {
                                    //No deberia entrar aca
                                    println!("{} {} {}", id, hostname, port);
                                }
                                commons::DistMsg::Ack => {
                                    println!("ACK")
                                }
                                commons::DistMsg::Nack => {
                                    println!("NACK")
                                }
                            },
                            Err(err) => println!("ERROR: {}", err),
                        };
                    }
                });
            }
        });

        self.join_handle = Some(t);
    }

    pub fn start_as_replic(self: &mut Self, replic_hostname: &str, replic_port: &str) {
        let h = self.hostname.clone();
        let p = self.port.clone();
        let connections = self.connections.clone();
        let pool = self.threadpool.clone();
        let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();

        let t = thread::spawn(move || {
    
            for stream in listener.incoming() {
                let pool2 = pool.clone();
                let connections_pool = connections.clone();
                pool2.execute(move || {
                    let mut reader = io::BufReader::new(stream.unwrap());
                    loop {
                        let mut s = String::new();

                        let len = match reader.read_line(&mut s) {
                            Ok(val) => val,
                            Err(_err) => 0,
                        };
                        if s.is_empty() || len == 0 {
                            return;
                        }
                        match commons::deserialize_dist(s.to_string()) {
                            Ok(val) => match val {
                                commons::DistMsg::Discover { id, hostname, port } => {
                                    //No deberia entrar aca
                                    println!("Discover: ID:{}, {}:{}", id, hostname, port);
                                }
                                commons::DistMsg::NewReplic { id, hostname, port } => {
                                    println!("{} {} {}", id, hostname, port);

                                    let newreplicstream =
                                        TcpStream::connect(format!("{}:{}", hostname, port))
                                            .unwrap();

                                    ({
                                        connections_pool.lock().unwrap().push((
                                            newreplicstream,
                                            id,
                                            hostname,
                                            port,
                                        ));
                                    });
                                }
                                commons::DistMsg::Ack => {
                                    println!("ACK")
                                }
                                commons::DistMsg::Nack => {
                                    println!("NACK")
                                }
                            },
                            Err(err) => println!("ERROR: {}", err),
                        };
                    }
                });
            }
        });

        self.join_handle = Some(t);

        //reach other replic to inform of
        let mut stream =
            TcpStream::connect(format!("{}:{}", replic_hostname, replic_port)).unwrap();
        stream
            .write(
                &(serde_json::to_string(&commons::DistMsg::Discover {
                    id: self.id,
                    hostname: self.hostname.to_string(),
                    port: self.port.to_string(),
                })
                .unwrap()
                    + "\n")
                    .as_bytes(),
            )
            .unwrap();
    }
    /*
        pub fn broadcast(&self, msg: commons::msg){

        }

        pub fn election() {
            for r in self.replics {
                r.send('ELECTIOn', self.i);
            }
        }
    */
}

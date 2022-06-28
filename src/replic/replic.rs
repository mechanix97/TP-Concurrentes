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
    //leader_hostname: String,
    //leader_port: String,
    //partners:
}

impl Replic {
    pub fn join(self: &mut Self) {
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
        }
    }

    pub fn start_as_leader(self: &mut Self) {
        self.is_leader = true;

        let h = self.hostname.clone();
        let p = self.port.clone();
        let connections = self.connections.clone();

        let t = thread::spawn(move || {
            let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();
            let pool = lib::ThreadPool::new(100);

            for stream in listener.incoming() {
                let connections_pool = connections.clone();
                pool.execute(move || {
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
                                    let mut nstream =
                                        TcpStream::connect(format!("{}:{}", hostname, port))
                                            .unwrap();
                                    nstream
                                        .write(
                                            &(serde_json::to_string(&commons::DistMsg::Ack)
                                                .unwrap()
                                                + "\n")
                                                .as_bytes(),
                                        )
                                        .unwrap();
                                    let h = hostname.clone();
                                    let p = port.clone();
                                        ({
                                        connections_pool
                                            .lock()
                                            .unwrap()
                                            .push((nstream, id, h, p));
                                    });

                                    let v = &mut *connections_pool.lock().unwrap();

                                    for conn in &mut *v {
                                        conn.0
                                            .write(
                                                &(serde_json::to_string(
                                                    &commons::DistMsg::NewReplic { id: id, hostname: hostname.clone(), port: port.clone() },
                                                )
                                                .unwrap()
                                                    + "\n")
                                                    .as_bytes(),
                                            ).unwrap();
                                    }
                                }
                                commons::DistMsg::NewReplic { id, hostname, port } => {
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

        let t = thread::spawn(move || {
            let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();
            let pool = lib::ThreadPool::new(100);
            for stream in listener.incoming() {
                pool.execute(|| {
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
                                    println!("Discover: ID:{}, {}:{}", id, hostname, port);
                                }
                                commons::DistMsg::NewReplic { id, hostname, port } => {
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

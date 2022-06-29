use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};

pub use crate::commons::*;
pub use crate::lib::*;

pub struct Replic {
    id: u32,
    hostname: String,
    port: String,
    is_leader: bool,
    join_handle: Option<JoinHandle<()>>,
    main_join_handle: Option<JoinHandle<()>>,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    threadpool: Arc<lib::ThreadPool>,
    leader_alive: Arc<AtomicBool>,
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
            main_join_handle: None,
            connections: Arc::new(Mutex::new(vec![])),
            threadpool: Arc::new(lib::ThreadPool::new(100)),
            leader_alive: Arc::new(AtomicBool::new(false))
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
                    let read_stream = stream.unwrap();
                    let mut write_stream = read_stream.try_clone().unwrap();
                    let mut reader = io::BufReader::new(read_stream);
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

                                    for conn in &mut *connections_pool.lock().unwrap() {
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
                                            false
                                        ));
                                    });
                                }
                                commons::DistMsg::NewReplic { id, hostname, port } => {
                                    //No deberia entrar aca
                                    println!("{} {} {}", id, hostname, port);
                                }
                                commons::DistMsg::Ping => {
                                    println!("ME LLEGA PING");
                                    write_stream.write(&(serde_json::to_string(&commons::DistMsg::Pong) 
                                        .unwrap()
                                            + "\n")
                                            .as_bytes(),
                                    )
                                    .unwrap();
                                }
                                commons::DistMsg::Pong => {
                                    println!("Pong")
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
        let leader_alive = self.leader_alive.clone();

        let t = thread::spawn(move || {
    
            for stream in listener.incoming() {
                let leader_alive2 = leader_alive.clone();
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
                                    leader_alive2.store(true, Ordering::Relaxed);
                                    let newreplicstream =
                                        TcpStream::connect(format!("{}:{}", hostname, port))
                                            .unwrap();

                                    ({
                                        connections_pool.lock().unwrap().push((
                                            newreplicstream,
                                            id,
                                            hostname,
                                            port,
                                            false
                                        ));
                                    });
                                }
                                commons::DistMsg::Ping => {
                                    println!("Ping")
                                }
                                commons::DistMsg::Pong => {
                                    leader_alive2.store(true, Ordering::Relaxed);
                                    println!("Pong")
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

        let mut stream =
            TcpStream::connect(format!("{}:{}", replic_hostname.clone(), replic_port.clone())).unwrap();
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

        let h = replic_hostname.clone();
        let r = replic_port.clone();
        {  
            //agrego conexion del leader a la lista (asumo id = 0 mas bajo posible)
            self.connections.lock().unwrap().push((stream, 0, h.to_string(), r.to_string(), true));
        }
        //replic main loop
        let main_leader_alive = self.leader_alive.clone();
        let connections = self.connections.clone();
        
        let mjh = thread::spawn(move || loop {
            main_leader_alive.store(false, Ordering::Relaxed);
            thread::sleep(time::Duration::from_secs(5));
            if !main_leader_alive.load(Ordering::Relaxed) {
                for c in &mut*connections.lock().unwrap() {
                    if c.4 {
                            c.0.write(&(serde_json::to_string(&commons::DistMsg::Ping) 
                            .unwrap()
                                + "\n")
                                .as_bytes(),
                        )
                        .unwrap();

                        c.0.set_read_timeout(Some(time::Duration::from_secs(5)));

                        let mut reader = io::BufReader::new(c.0.try_clone().unwrap());

                        let mut s = String::new();

                        let _ = match reader.read_line(&mut s) {
                            Ok(val) => val,
                            Err(_err) => 0,
                        };
                        match commons::deserialize_dist(s.to_string())  {
                            Ok(val) => match val {
                                commons::DistMsg::Pong => {
                                    println!("PONG");
                                    main_leader_alive.store(true, Ordering::Relaxed);
                                }
                                _ => {}
                            },
                            Err(_) => {}
                        }
                        if !main_leader_alive.load(Ordering::Relaxed) {
                            println!("NO RESPONSE");
                            //NO HUBO RESPUESTA DEL LIDER
                            //HACER ELECTION
                        }
                    }                  
                }
            }

        });
        self.main_join_handle = Some(mjh);
    }

}

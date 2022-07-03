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
    is_leader: Arc<Mutex<bool>>,
    join_handle: Option<JoinHandle<()>>,
    main_join_handle: Option<JoinHandle<()>>,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    threadpool: Arc<lib::ThreadPool>,
    leader_alive: Arc<AtomicBool>,
}

impl Replic {
    pub fn join(self: &mut Self) {
        //No anda :/
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
            is_leader: Arc::new(Mutex::new(false)),
            join_handle: None,
            main_join_handle: None,
            connections: Arc::new(Mutex::new(vec![])),
            threadpool: Arc::new(lib::ThreadPool::new(100)),
            leader_alive: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn start_as_leader(self: &mut Self) {
        {
            *self.is_leader.lock().unwrap() = true;
        }
        self.start();

        let mjh = thread::spawn(move || loop {
            //Read input file
            //broadcast commit when operation is ended correct
            //broadcast rollback when operation fails
        });

        self.main_join_handle = Some(mjh);
    }

    pub fn start_as_replic(self: &mut Self, replic_hostname: &str, replic_port: &str) {
        self.start();      

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
        
        let mjh = thread::spawn(move || loop{
            check_leader_alive(connections.clone(), main_leader_alive.clone());
            if election(connections.clone()){
                break;
            }

        });

        self.main_join_handle = Some(mjh);
    }

    pub fn start(&mut self) {
        let h = self.hostname.clone();
        let p = self.port.clone();
        let is_leader = self.is_leader.clone();
        let connections = self.connections.clone();
        let pool = self.threadpool.clone();
        let leader_alive = self.leader_alive.clone();
        
        let t = thread::spawn(move || {
            let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();
            for stream in listener.incoming() {
                let connections_pool = connections.clone();
                let la = leader_alive.clone();
                let il = is_leader.clone();
                pool.execute(move || {
                    let read_stream = stream.unwrap();
                    let mut writer = read_stream.try_clone().unwrap();
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

                        let mut is_leader = false;
                        {
                            is_leader = *il.lock().unwrap();
                        }

                        match commons::deserialize_dist(s.to_string()) {
                            Ok(val) => match val {
                                commons::DistMsg::Discover { id, hostname, port } => {
                                    if is_leader {
                                        leader_msg_discover(id, hostname, port, connections_pool.clone());
                                    } else {
                                        replic_msg_discover(id, hostname, port);
                                    }                                   
                                }
                                commons::DistMsg::NewReplic { id, hostname, port } => {
                                    if is_leader {
                                        leader_msg_new_replic(id, hostname, port);
                                    } else {
                                        replic_msg_new_replic(id, hostname, port, connections_pool.clone(), la.clone());
                                    }
                                }
                                commons::DistMsg::Election {id} => {
                                    println!("Election id:{}", id);
                                }
                                commons::DistMsg::Ping => {
                                    writer.write(&(serde_json::to_string(&commons::DistMsg::Pong) 
                                        .unwrap()
                                            + "\n")
                                            .as_bytes(),
                                    )
                                    .unwrap();
                                }
                                commons::DistMsg::Pong => {
                                    println!("Pong")
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
    
}

fn leader_msg_discover(id: u32,
        hostname: String,
        port: String, 
        connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) {
    //conexion a replica entrante
    let mut newreplicstream =
    TcpStream::connect(format!("{}:{}", hostname, port))
        .unwrap();

    for conn in &mut *connections.lock().unwrap() {
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
        connections.lock().unwrap().push((
            newreplicstream,
            id,
            hostname,
            port,
            false
        ));
    });
}

fn leader_msg_new_replic(id: u32,
    hostname: String,
    port: String){
    //No deberia entrar aca
    println!("{} {} {}", id, hostname, port);
} 

fn replic_msg_discover(id: u32,
    hostname: String,
    port: String) {
    //No deberia entrar aca
    println!("Discover: ID:{}, {}:{}", id, hostname, port);
}

fn replic_msg_new_replic(id: u32,
    hostname: String,
    port: String,
    connections:  Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    leader_alive: Arc<AtomicBool>){
    leader_alive.store(true, Ordering::Relaxed);
    let newreplicstream =
    TcpStream::connect(format!("{}:{}", hostname, port))
        .unwrap();

    ({
        connections.lock().unwrap().push((
            newreplicstream,
            id,
            hostname,
            port,
            false
        ));
    });
}

fn check_leader_alive(connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>, main_leader_alive: Arc<AtomicBool>) {
    loop {
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
                                main_leader_alive.store(true, Ordering::Relaxed);
                            }
                            _ => {}
                        },
                        Err(_) => {}
                    }
                    if !main_leader_alive.load(Ordering::Relaxed) {  
                        return;
                    }
                }                  
            }
        }
    }
}

/// Return True if replic is new leader
fn election(connections:  Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) -> bool{
    //Elimino al leader de la lista de conexiones
    let index =  connections.lock().unwrap().iter().position(|c| c.4).unwrap();
    connections.lock().unwrap().remove(index);   
    
    for c in &mut*connections.lock().unwrap() {
        c.0.write(&(serde_json::to_string(&commons::DistMsg::Election{id: 0}) 
            .unwrap()
                + "\n")
                .as_bytes(),
        )
        .unwrap();
    }
    println!("HOLA");
    return true;
}


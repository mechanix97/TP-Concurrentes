use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::thread;

use crate::leader;
use crate::replic;

pub use crate::actor_airline::{AirlineActor, FlightPrice};
pub use crate::actor_bank::{BankActor, PaymentPrice};
pub use crate::actor_hotel::{HotelActor, ReservationPrice};
pub use crate::commons::{deserialize_dist, DistMsg};
pub use crate::lib::*;

pub struct Alglobo {
    id: u32,
    hostname: String,
    port: String,
    is_leader: Arc<Mutex<bool>>,
    join_handle: Option<JoinHandle<()>>,
    main_join_handle: Option<JoinHandle<()>>,
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    threadpool: Arc<ThreadPool>,
    leader_alive: Arc<AtomicBool>,
    leader_ok: Arc<(Mutex<bool>, Condvar)>,
}

impl Alglobo {
    pub fn join(self: &mut Self) {
        drop(&self.threadpool);

        match self.join_handle.take() {
            Some(h) => {
                h.join().unwrap();
            }
            None => {}
        }
    }

    pub fn new(id: u32, hostname: &str, port: &str) -> Alglobo {
        Alglobo {
            id: id,
            hostname: hostname.to_string(),
            port: port.to_string(),
            is_leader: Arc::new(Mutex::new(false)),
            join_handle: None,
            main_join_handle: None,
            connections: Arc::new(Mutex::new(vec![])),
            threadpool: Arc::new(ThreadPool::new(100)),
            leader_alive: Arc::new(AtomicBool::new(false)),
            leader_ok: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn start_as_leader(self: &mut Self) {
        {
            *self.is_leader.lock().unwrap() = true;
        }
        self.start();

        let connections = self.connections.clone();
        let id = self.id;
        let mjh = thread::spawn(move || leader::main_loop::exec(id, connections));

        self.main_join_handle = Some(mjh);
    }

    pub fn start_as_replic(self: &mut Self, hostname: String, port: String) {
        self.start();

        self.connect(hostname, port);


        let id = self.id;
        let connections = self.connections.clone();
        let main_leader_alive = self.leader_alive.clone();
        let is_leader = self.is_leader.clone();
        let leader_ok = self.leader_ok.clone();

        //replic main loop
        let mjh = thread::spawn(move || replic::main_loop::exec(id, connections, main_leader_alive, is_leader, leader_ok));

        self.main_join_handle = Some(mjh);
    }

    pub fn start(&mut self) {
        let id = self.id;
        let h = self.hostname.clone();
        let p = self.port.clone();
        let is_leader = self.is_leader.clone();
        let connections = self.connections.clone();
        let pool = self.threadpool.clone();
        let leader_alive = self.leader_alive.clone();
        let leader_ok = self.leader_ok.clone();

        let t = thread::spawn(move || {
            let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();
            for stream in listener.incoming() {
                let mid = id;
                let connections_pool = connections.clone();
                let la = leader_alive.clone();
                let il = is_leader.clone();
                let lo = leader_ok.clone();

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

                        let is_leader;
                        {
                            is_leader = *il.lock().unwrap();
                        }

                        match deserialize_dist(s.to_string()) {
                            Ok(val) => match val {
                                DistMsg::Discover { id, hostname, port } => {
                                    if is_leader {
                                        leader::msg_discover::exec(
                                            id,
                                            hostname,
                                            port,
                                            connections_pool.clone(),
                                        );
                                    } else {
                                        replic::msg_discover::exec(id, hostname, port);
                                    }
                                }
                                DistMsg::NewReplic { id, hostname, port } => {
                                    if is_leader {
                                        leader::msg_new_replic::exec(id, hostname, port);
                                    } else {
                                        replic::msg_new_replic::exec(
                                            id,
                                            hostname,
                                            port,
                                            connections_pool.clone(),
                                            la.clone(),
                                        );
                                    }
                                }
                                DistMsg::Election { id: _ } => {
                                    replic::msg_election::exec(writer.try_clone().unwrap(), mid);
                                }
                                DistMsg::Leader { id } => {
                                    for c in &mut *connections_pool.lock().unwrap() {
                                        if c.1 == id {
                                            c.4 = true;
                                        }
                                    }
                                    let (lock, cvar) = &*lo;
                                    let mut started = lock.lock().unwrap();
                                    *started = true;
                                    cvar.notify_all();
                                }
                                DistMsg::Commit { transaction } => {
                                    println!("recibo commit: {}", transaction);
                                }
                                DistMsg::Rollback { transaction } => {
                                    println!("recibo rollback: {}", transaction);
                                }
                                DistMsg::Ping => {
                                    writer
                                        .write(
                                            &(serde_json::to_string(&DistMsg::Pong).unwrap()
                                                + "\n")
                                                .as_bytes(),
                                        )
                                        .unwrap();
                                }
                                DistMsg::Pong => {
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

    fn connect(&mut self, replic_hostname: String, replic_port: String) {
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            replic_hostname.clone(),
            replic_port.clone()
        ))
        .unwrap();
        stream
            .write(
                &(serde_json::to_string(&DistMsg::Discover {
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
            self.connections
                .lock()
                .unwrap()
                .push((stream, 0, h.to_string(), r.to_string(), true));
        }
    }
}


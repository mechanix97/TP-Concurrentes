use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::leader;
use crate::replic;
use crate::common;

pub use crate::commons::{deserialize_dist, DistMsg};
pub use crate::lib::*;
pub use crate::logger::*;
pub use crate::transaction_writer::*;
pub use crate::connection::*;

pub struct Alglobo {
    id: u32,
    hostname: String,
    port: String,
    is_leader: Arc<Mutex<bool>>,
    join_handle: Option<JoinHandle<()>>,
    main_join_handle: Option<JoinHandle<()>>,
    connections: Arc<Mutex<Vec<Connection>>>,
    threadpool: Arc<ThreadPool>,
    leader_alive: Arc<AtomicBool>,
    leader_ok: Arc<(Mutex<bool>, Condvar)>,
    listener: Option<TcpListener>,
    running: Arc<Mutex<bool>>
}

impl Alglobo {
    pub fn join(self: &mut Self) {
        {
            *self.running.lock().unwrap() = false;
        }
        for c in &mut *self.connections.lock().unwrap() {
            c.write(DistMsg::Shutdown {
                    hostname: self.hostname.clone(),
                    port: self.port.clone(),
                    shutdown: false,
                }
                
            )
            
        }
        drop(&self.threadpool);
        match &self.listener {
            Some(l) => l.set_nonblocking(true).unwrap(),
            None => {}
        }
        TcpStream::connect(format!("{}:{}", self.hostname.clone(), self.port.clone())).unwrap();
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
            listener: None,
            running: Arc::new(Mutex::new(true)),
        }
    }

    pub fn start_as_leader(self: &mut Self) {
        let logger = Logger::new(format!("log/replic_{}.txt", self.id));
        let commiter = TransactionWriter::new(format!("transactions/commits{}.txt", self.id));
        let rollbacker = TransactionWriter::new(format!("transactions/rollbacks{}.txt", self.id));
        {
            *self.is_leader.lock().unwrap() = true;
        }
        self.start(logger.clone(), commiter.clone(), rollbacker.clone());

        let connections = self.connections.clone();
        let id = self.id;
        let running = self.running.clone();
        let mjh = thread::spawn(move || leader::main_loop::exec(id, connections, logger.clone(), commiter.clone(), rollbacker.clone(), running));

        self.main_join_handle = Some(mjh);
    }

    pub fn start_as_replic(self: &mut Self, hostname: String, port: String) {
        let logger = Logger::new(format!("log/replic_{}.txt", self.id));
        let commiter = TransactionWriter::new(format!("transactions/commits{}.txt", self.id));
        let rollbacker = TransactionWriter::new(format!("transactions/rollbacks{}.txt", self.id));
        self.start(logger.clone(), commiter.clone(), rollbacker.clone());

        self.connect(hostname, port);

        let id = self.id;
        let connections = self.connections.clone();
        let main_leader_alive = self.leader_alive.clone();
        let is_leader = self.is_leader.clone();
        let leader_ok = self.leader_ok.clone();
        let running = self.running.clone();
        //replic main loop
        logger.log(format!("Starting replic loop"));
        let mjh = thread::spawn(move || {
            replic::main_loop::exec(
                id,
                connections,
                main_leader_alive,
                is_leader,
                leader_ok,
                logger.clone(),
                commiter.clone(), 
                rollbacker.clone(),
                running
            )
        });

        self.main_join_handle = Some(mjh);
    }

    pub fn start(&mut self, logger: Logger, commiter: TransactionWriter, 
        rollbacker: TransactionWriter) {
        let id = self.id;
        let h = self.hostname.clone();
        let p = self.port.clone();
        let is_leader = self.is_leader.clone();
        let connections = self.connections.clone();
        let pool = self.threadpool.clone();
        let leader_alive = self.leader_alive.clone();
        let leader_ok = self.leader_ok.clone();

        let listener = TcpListener::bind(format!("{}:{}", h, p)).unwrap();
        self.listener = Some(listener.try_clone().unwrap());

        let t = thread::spawn(move || loop {
            let stream = match listener.accept() {
                Ok(v) => v.0,
                Err(_) => break,
            };

            let mid = id;
            let hostname = h.clone();
            let port = p.clone();
            let connections_pool = connections.clone();
            let la = leader_alive.clone();
            let il = is_leader.clone();
            let lo = leader_ok.clone();
            let logger_pool = logger.clone();
            let commiter_pool = commiter.clone();
            let rollbacker_pool = rollbacker.clone();

            pool.execute(move || {
                let read_stream = stream;
                let hostname_pool = hostname.clone();
                let port_pool = port.clone();
                let mut writer = read_stream.try_clone().unwrap();
                let mut reader = io::BufReader::new(read_stream);
                let remote_host = writer.peer_addr().unwrap().ip();
                let remote_port = writer.peer_addr().unwrap().port();
                logger_pool.log(format!(
                    "new connection from {}:{}",
                    remote_host, remote_port
                ));
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
                            DistMsg::Discover { id, hostname, port , is_leader: _} => {
                                common::msg_discover::exec(
                                    id,
                                    hostname,
                                    port,
                                    connections_pool.clone(),
                                    logger_pool.clone()
                                );
                                writer.write(&DistMsg::Discover{id: mid, hostname: hostname_pool.clone(), port: port_pool.clone(), is_leader: is_leader}.to_string().as_bytes()).unwrap();
                            }
                            DistMsg::NewReplic { id, hostname, port } => {   
                                common::msg_new_replic::exec(
                                    id,
                                    hostname,
                                    port,
                                    connections_pool.clone(),
                                    logger_pool.clone()
                                );                                
                            }
                            DistMsg::Election { id: _ } => {
                                replic::msg_election::exec(writer.try_clone().unwrap(), mid, remote_host.to_string(), remote_port.to_string(), logger_pool.clone());
                            }
                            DistMsg::NewLeader { id } => {
                                logger_pool.log(format!(
                                    "received NewLeader from {}:{}",
                                    remote_host, remote_port
                                ));

                                for c in &mut *connections_pool.lock().unwrap() {
                                    if c.get_id() == id {
                                        c.set_as_leader();
                                    }
                                }
                                let (lock, cvar) = &*lo;
                                let mut started = lock.lock().unwrap();
                                *started = true;
                                cvar.notify_all();
                                println!("SOY REPLICA");
                            }
                            DistMsg::Commit { transaction } => {
                                common::msg_commit::exec(logger_pool.clone(), commiter_pool.clone(), remote_host.to_string(), remote_port.to_string(), transaction.clone(), la.clone());
                            }
                            DistMsg::Rollback { transaction } => {
                                common::msg_rollback::exec(logger_pool.clone(), rollbacker_pool.clone(), remote_host.to_string(), remote_port.to_string(), transaction.clone(), la.clone());
                            }
                            DistMsg::Ping => {
                                logger_pool.log(format!(
                                    "received Ping from {}:{}",
                                    remote_host, remote_port
                                ));

                                writer.write(&DistMsg::Pong.to_string().as_bytes()).unwrap();
                            }
                            DistMsg::Pong => {
                                logger_pool.log(format!(
                                    "received Pong from {}:{}",
                                    remote_host, remote_port
                                ));

                                println!("Pong");
                                break;
                            }
                            DistMsg::Shutdown {
                                hostname,
                                port,
                                shutdown,
                            } => {
                                logger_pool.log(format!(
                                    "received Shutdown from {}:{}",
                                    remote_host, remote_port
                                ));

                                if shutdown {
                                    logger_pool.log(format!(
                                        "shutting down"
                                    ));
                                    break;
                                } else {
                                    replic::msg_shutdown::exec(
                                        hostname,
                                        port,
                                        connections_pool.clone(),
                                    );
                                }
                            }
                        },
                        Err(err) => logger_pool.log(format!(
                            "Error from {}:{}, ERROR: {}",
                            remote_host, remote_port, err
                        )),
                    };
                }
            });
        });

        self.join_handle = Some(t);
    }

    fn connect(&mut self, replic_hostname: String, replic_port: String) {
        let mut newconnetion = Connection::new(0, replic_hostname, replic_port);
 
        newconnetion.write(DistMsg::Discover {
            id: self.id,
            hostname: self.hostname.to_string(),
            port: self.port.to_string(),
            is_leader: false
        });

        let mut reader = io::BufReader::new( newconnetion.get_stream());
        let mut s = String::new();

        let len = match reader.read_line(&mut s) {
            Ok(val) => val,
            Err(_err) => 0,
        };
        if s.is_empty() || len == 0 {
            return;
        }
        let nc = &mut newconnetion;

        match deserialize_dist(s).unwrap(){
            DistMsg::Discover{id, hostname: _, port: _, is_leader} => {
                nc.set_id(id);
                if is_leader {
                    nc.set_as_leader();
                }
            }
            _ => return
        }

        {   //agrego conexion del leader a la lista (asumo id = 0 mas bajo posible)
            self.connections
                .lock()
                .unwrap()
                .push(newconnetion);
        }
    }
}

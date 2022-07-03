use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, Condvar};
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
    leader_ok: Arc<(Mutex<bool>, Condvar)>
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
            leader_alive: Arc::new(AtomicBool::new(false)),
            leader_ok: Arc::new((Mutex::new(true), Condvar::new()))
        }
    }

    pub fn start_as_leader(self: &mut Self) {
        {
            *self.is_leader.lock().unwrap() = true;
        }
        self.start();

        let connections = self.connections.clone();
        let id = self.id;
        let mjh = thread::spawn(move || leader_main_loop(id, connections));

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
        let id = self.id;
        let is_leader = self.is_leader.clone();
        let leader_ok = self.leader_ok.clone();
        
        let mjh = thread::spawn(move || loop{
            check_leader_alive(connections.clone(), main_leader_alive.clone());
            if election(id, connections.clone()){
                //leader
                *is_leader.lock().unwrap() = true;
                leader_main_loop(id, connections.clone());
            } else {
                //replic
                println!("ESPERO AL NUEVO LIDER");
                wait_for_leader(leader_ok.clone());
                println!("TENGO NUEVO LIDER");
            }

        });

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
                                    writer.write(&(serde_json::to_string(&commons::DistMsg::Election{id: mid}).unwrap()
                                            + "\n")
                                            .as_bytes(),
                                    )
                                    .unwrap();
                                }
                                commons::DistMsg::Leader {id} => {
                                    println!("LIDER RECIBIDO: {}", id);
                                    for c in &mut*connections_pool.lock().unwrap(){
                                        if c.1 == id {
                                            c.4 = true;
                                        }
                                    }
                                    let (lock, cvar) = &*lo;
                                    let mut started = lock.lock().unwrap();
                                    *started = true;
                                    cvar.notify_all();
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
        connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) 
{
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
    port: String)
{
    //No deberia entrar aca
    println!("{} {} {}", id, hostname, port);
} 

fn replic_msg_discover(id: u32,
    hostname: String,
    port: String) 
{
    //No deberia entrar aca
    println!("Discover: ID:{}, {}:{}", id, hostname, port);
}

fn replic_msg_new_replic(id: u32,
    hostname: String,
    port: String,
    connections:  Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    leader_alive: Arc<AtomicBool>)
{
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

fn check_leader_alive(
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>, 
    main_leader_alive: Arc<AtomicBool>) 
{
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

                    c.0.set_read_timeout(Some(time::Duration::from_secs(5))).unwrap();

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
fn election(
    mid: u32, 
    connections:  Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) 
-> bool {
    //Elimino al leader de la lista de conexiones
    let index =  connections.lock().unwrap().iter().position(|c| c.4).unwrap();
    connections.lock().unwrap().remove(index);   
    
    let mut iamleader = true;

    for c in &mut*connections.lock().unwrap() {
        c.0.write(&(serde_json::to_string(&commons::DistMsg::Election{id: mid}) 
            .unwrap()
                + "\n")
                .as_bytes(),
        )
        .unwrap();
        c.0.set_read_timeout(Some(time::Duration::from_secs(5))).unwrap();
        let mut reader = io::BufReader::new(c.0.try_clone().unwrap());
        let mut s = String::new();

        let _ = match reader.read_line(&mut s) {
            Ok(val) => val,
            Err(_err) => 0,
        };
        match commons::deserialize_dist(s.to_string())  {
            Ok(val) => match val {
                commons::DistMsg::Election { id } => {
                    if id < mid {
                        iamleader = false;
                    }
                }
                _ => {}
            },
            Err(_) => {}
        }
    }
    if iamleader{
        println!("SOY LIDER");
    } else {
        println!("SOY REPLICA");
    }
    return iamleader;
}

fn leader_main_loop(id: u32, connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) {
    for c in &mut*connections.lock().unwrap() {
        c.0.write(&(serde_json::to_string(&commons::DistMsg::Leader{id: id}) 
            .unwrap()
                + "\n")
                .as_bytes(),
        )
        .unwrap();
    }
    loop {
        //Read input file
        //broadcast commit when operation is ended correct
        //broadcast rollback when operation fails
    }
}

fn wait_for_leader(pair: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*pair;
    let mut ok = lock.lock().unwrap();
    *ok = false;
    while !*ok {
        ok = cvar.wait(ok).unwrap();
    }
}
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, Condvar};
use std::{thread, time};
use std::thread::{JoinHandle, sleep};
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs;
use actix::{prelude::*};
use futures::join;
use chrono::Local;

pub use crate::commons::commons::{deserialize_dist, DistMsg};
pub use crate::lib::*;

pub use crate::hotel::{HotelActor, ReservationPrice};
pub use crate::bank::{BankActor, PaymentPrice};
pub use crate::airline::{FlightPrice, AirlineActor};



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
            leader_ok: Arc::new((Mutex::new(false), Condvar::new()))
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
                wait_for_leader(leader_ok.clone());
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

                        let is_leader;
                        {
                            is_leader = *il.lock().unwrap();
                        }

                        match deserialize_dist(s.to_string()) {
                            Ok(val) => match val {
                                DistMsg::Discover { id, hostname, port } => {
                                    if is_leader {
                                        leader_msg_discover(id, hostname, port, connections_pool.clone());
                                    } else {
                                        replic_msg_discover(id, hostname, port);
                                    }                                   
                                }
                                DistMsg::NewReplic { id, hostname, port } => {
                                    if is_leader {
                                        leader_msg_new_replic(id, hostname, port);
                                    } else {
                                        replic_msg_new_replic(id, hostname, port, connections_pool.clone(), la.clone());
                                    }
                                }
                                DistMsg::Election {id: _} => {
                                    writer.write(&(serde_json::to_string(&DistMsg::Election{id: mid}).unwrap()
                                            + "\n")
                                            .as_bytes(),
                                    )
                                    .unwrap();
                                }
                                DistMsg::Leader {id} => {
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
                                DistMsg::Commit {transaction} => {
                                    println!("recibo commit: {}", transaction);
                                }
                                DistMsg::Rollback {transaction} => {
                                    println!("recibo rollback: {}", transaction);
                                }
                                DistMsg::Ping => {
                                    writer.write(&(serde_json::to_string(&DistMsg::Pong) 
                                        .unwrap()
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
                &DistMsg::NewReplic {
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
                &DistMsg::NewReplic {
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
                        c.0.write(&(serde_json::to_string(&DistMsg::Ping) 
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
                    match deserialize_dist(s.to_string())  {
                        Ok(val) => match val {
                            DistMsg::Pong => {
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
    
    let mut amileader = true;

    for c in &mut*connections.lock().unwrap() {
        c.0.write(&(serde_json::to_string(&DistMsg::Election{id: mid}) 
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
        match deserialize_dist(s.to_string())  {
            Ok(val) => match val {
                DistMsg::Election { id } => {
                    if id < mid {
                        amileader = false;
                    }
                }
                _ => {}
            },
            Err(_) => {}
        }
    }
    return amileader;
}

fn leader_main_loop(id: u32, connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>) {
    for c in &mut*connections.lock().unwrap() {
        c.0.write(&(serde_json::to_string(&DistMsg::Leader{id: id}) 
            .unwrap()
                + "\n")
                .as_bytes(),
        )
        .unwrap();
    }
    println!("SOY LIDER");
    sleep(time::Duration::from_secs(10));
    let sys = actix::System::new();

    println!("{}:INICIO", Local::now().format("%Y-%m-%d %H:%M:%S"));
    let filename = r"transactions.txt";
    let file = fs::File::open(filename).expect("Error: file not found!");
    let  buf_reader =  io::BufReader::new(file);

    sys.block_on(async {
        
        let addr_bank = BankActor { bank_connection: TcpStream::connect("127.0.0.1:7878").unwrap() }.start();
        let addr_hotel = HotelActor { hotel_connection: TcpStream::connect("127.0.0.1:7879").unwrap() }.start();
        let addr_airline = AirlineActor { airline_connection: TcpStream::connect("127.0.0.1:7880").unwrap() }.start();

        for line in buf_reader.lines() {
            let transaction = line.unwrap();            
            
            let separated_line: Vec<&str> = transaction.split(',').collect();
    
            let transaction_id = separated_line[0].trim().to_string().parse::<i32>().unwrap();
            let bank_payment = separated_line[1].trim().to_string().parse::<f32>().unwrap();
            let hotel_payment = separated_line[2].trim().to_string().parse::<f32>().unwrap();
            let airline_payment = separated_line[3].trim().to_string().parse::<f32>().unwrap();

            let result_hotel = addr_hotel.send(ReservationPrice(transaction_id, hotel_payment));

            let result_bank = addr_bank.send(PaymentPrice(transaction_id, bank_payment));

            let result_airline = addr_airline.send(FlightPrice(transaction_id, airline_payment)); 

            
            let res = join!(result_hotel, result_bank, result_airline ); //Resultado
            let result0 =match res.0 {
                Ok(val) => {match val{
                    Ok(val) => val,
                    Err(_) => false
                }}
                Err(_) => false
            };
            let result1 =match res.1 {
                Ok(val) => {match val{
                    Ok(val) => val,
                    Err(_) => false
                }}
                Err(_) => false
            };
            let result2 =match res.2 {
                Ok(val) => {match val{
                    Ok(val) => val,
                    Err(_) => false
                }}
                Err(_) => false
            };

            if result0 && result1 && result2 {
                for c in &mut*connections.lock().unwrap() {
                    c.0.write(&(serde_json::to_string(&DistMsg::Commit{transaction: transaction.clone()}) 
                        .unwrap()
                            + "\n")
                            .as_bytes(),
                    )
                    .unwrap();
                }
            } else {
                for c in &mut*connections.lock().unwrap() {
                    c.0.write(&(serde_json::to_string(&DistMsg::Rollback{transaction: transaction.clone()}) 
                        .unwrap()
                            + "\n")
                            .as_bytes(),
                    )
                    .unwrap();
                }
            }
        }
    });
    
    System::current().stop();
    sys.run().unwrap();
    
    println!("{}: TERMINO TRANSACCIONES", Local::now().format("%Y-%m-%d %H:%M:%S"));

}

fn wait_for_leader(pair: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*pair;
    let mut ok = lock.lock().unwrap();
    while !*ok {
        ok = cvar.wait(ok).unwrap();
    }
    *ok = false;
}
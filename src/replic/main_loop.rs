use core::time;
use std::io::{self, BufRead};
use std::sync::atomic::AtomicBool;
use std::sync::Condvar;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::thread;

use crate::leader;
pub use crate::logger::*;
pub use crate::transaction_writer::*;
pub use crate::commons::{deserialize_dist, DistMsg};
pub use crate::connection::*;

pub fn exec(
    id: u32,
    connections: Arc<Mutex<Vec<Connection>>>,
    main_leader_alive: Arc<AtomicBool>,
    is_leader: Arc<Mutex<bool>>,
    leader_ok: Arc<(Mutex<bool>, Condvar)>,
    logger: Logger,
    commiter: TransactionWriter, 
    rollbacker: TransactionWriter,
    running: Arc<Mutex<bool>>
) {
    loop {
        check_leader_alive(connections.clone(), main_leader_alive.clone());
        if election(id, connections.clone()) {
            //leader
            *is_leader.lock().unwrap() = true;
            leader::main_loop::exec(id, connections.clone(), logger.clone(), commiter.clone(), rollbacker.clone(), running.clone());
            break;
        } else {
            //replic
            wait_for_leader(leader_ok.clone());
        }
    }
}

fn check_leader_alive(
    connections: Arc<Mutex<Vec<Connection>>>,
    main_leader_alive: Arc<AtomicBool>,
) {
    loop {
        main_leader_alive.store(false, Ordering::Relaxed);
        thread::sleep(time::Duration::from_secs(5));
        if !main_leader_alive.load(Ordering::Relaxed) {
            for c in &mut *connections.lock().unwrap() {
                if c.is_leader() {
                    c.write(DistMsg::Ping);
                    let s = c.get_stream();
                    s.set_read_timeout(Some(time::Duration::from_secs(5))).unwrap();

                    let mut reader = io::BufReader::new(s);
                    let mut s = String::new();

                    let _ = match reader.read_line(&mut s) {
                        Ok(val) => val,
                        Err(_err) => 0,
                    };
                    match deserialize_dist(s.to_string()) {
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
    connections: Arc<Mutex<Vec<Connection>>>,
) -> bool {
    //Elimino al leader de la lista de conexiones
    let index = connections
        .lock()
        .unwrap()
        .iter()
        .position(|c| c.is_leader())
        .unwrap();
    {
        connections.lock().unwrap().remove(index);    
    }    

    let mut amileader = true;

    for c in &mut *connections.lock().unwrap() {
        c.write(DistMsg::Election { id: mid });
        let s = c.get_stream();
        s.set_read_timeout(Some(time::Duration::from_secs(5))).unwrap();
        let mut reader = io::BufReader::new(s);
        let mut s = String::new();

        let _ = match reader.read_line(&mut s) {
            Ok(val) => val,
            Err(_err) => 0,
        };
        match deserialize_dist(s.to_string()) {
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

fn wait_for_leader(pair: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*pair;
    let mut ok = lock.lock().unwrap();
    while !*ok {
        ok = cvar.wait(ok).unwrap();
    }
    *ok = false;
}

use core::time;
use std::io::{Write, self, BufRead};
use std::net::TcpStream;
use std::sync::Condvar;
use std::sync::atomic::AtomicBool;
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::thread;

use crate::leader;

pub use crate::commons::{DistMsg, deserialize_dist};

pub fn exec(id: u32, connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>, main_leader_alive: Arc<AtomicBool>, is_leader: Arc<Mutex<bool>>, leader_ok: Arc<(Mutex<bool>, Condvar)>){
    loop {
        check_leader_alive(connections.clone(), main_leader_alive.clone());
        if election(id, connections.clone()) {
            //leader
            *is_leader.lock().unwrap() = true;
            leader::main_loop::exec(id, connections.clone());
        } else {
            //replic
            wait_for_leader(leader_ok.clone());
        }
    }   
}

fn check_leader_alive(
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
    main_leader_alive: Arc<AtomicBool>,
) {
    loop {
        main_leader_alive.store(false, Ordering::Relaxed);
        thread::sleep(time::Duration::from_secs(5));
        if !main_leader_alive.load(Ordering::Relaxed) {
            for c in &mut *connections.lock().unwrap() {
                if c.4 {
                    c.0.write(&(serde_json::to_string(&DistMsg::Ping).unwrap() + "\n").as_bytes())
                        .unwrap();

                    c.0.set_read_timeout(Some(time::Duration::from_secs(5)))
                        .unwrap();

                    let mut reader = io::BufReader::new(c.0.try_clone().unwrap());
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
    connections: Arc<Mutex<Vec<(TcpStream, u32, String, String, bool)>>>,
) -> bool {
    //Elimino al leader de la lista de conexiones
    let index = connections
        .lock()
        .unwrap()
        .iter()
        .position(|c| c.4)
        .unwrap();
    connections.lock().unwrap().remove(index);

    let mut amileader = true;

    for c in &mut *connections.lock().unwrap() {
        c.0.write(
            &(serde_json::to_string(&DistMsg::Election { id: mid }).unwrap() + "\n").as_bytes(),
        )
        .unwrap();
        c.0.set_read_timeout(Some(time::Duration::from_secs(5)))
            .unwrap();
        let mut reader = io::BufReader::new(c.0.try_clone().unwrap());
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

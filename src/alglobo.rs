use std::thread;
use std::net::{UdpSocket};
use std::time::Duration;
use std::env;
use rand::{Rng, thread_rng};

mod bully;
use bully::LeaderElection;

const TIMEOUT: Duration = Duration::from_secs(10);

fn id_to_dataaddr(id: usize) -> String { "127.0.0.1:1235".to_owned() + &*id.to_string() }

fn main() {
    let id = env::args().nth(1).unwrap();
    let id: usize = id.parse().unwrap();

    println!("[{}] iniciando", id);
    team_member(id);
}

fn team_member(id: usize) {

    loop {
        let mut scrum_master = LeaderElection::new(id);
        let socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
        let mut buf = [0; 4];

        loop {
            if scrum_master.am_i_leader() {
                println!("[{}] soy SM", id);
                if thread_rng().gen_range(0, 100) >= 90 {
                    println!("[{}] me tomo vacaciones", id);
                    break;
                }
                socket.set_read_timeout(None).unwrap();
                let (_, from) = socket.recv_from(&mut buf).unwrap();
                socket.send_to("PONG".as_bytes(), from).unwrap();
            } else {
                let leader_id = scrum_master.get_leader_id();
                println!("[{}] pido trabajo al SM {}", id, leader_id);
                socket.send_to("PING".as_bytes(), id_to_dataaddr(leader_id)).unwrap();
                socket.set_read_timeout(Some(TIMEOUT)).unwrap();
                if let Ok((_, _)) = socket.recv_from(&mut buf) {
                    println!("[{}] trabajando", id);
                    thread::sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000)));
                } else {
                    // por simplicidad consideramos que cualquier error necesita un lider nuevo
                    scrum_master.find_new()
                }
            }
        }

        scrum_master.stop();
        thread::sleep(Duration::from_secs(5));
    }
}

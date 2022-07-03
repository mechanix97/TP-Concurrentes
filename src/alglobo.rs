use std::thread;
use std::net::{UdpSocket};
use std::time::Duration;
use std::env;
use rand::{Rng, thread_rng};

mod bully;
use bully::LeaderElection;

const TIMEOUT: Duration = Duration::from_secs(4);

fn id_to_dataaddr(id: usize) -> String { "127.0.0.1:1235".to_owned() + &*id.to_string() }

fn main() {
    let count = env::args().nth(1).unwrap();
    let count: usize = count.parse().unwrap();
    let mut handles = vec!();

    for id in 0..count {
        handles.push(thread::spawn(move || { team_member(id, count) }));
    }
    handles.into_iter().for_each(|h| { h.join(); });
}

fn team_member(id: usize, count: usize) {

    println!("[{}] iniciando", id);
    loop {
        let mut scrum_master = LeaderElection::new(id, count);
        let socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
        let mut buf = [0; 4];

        loop {
            if thread_rng().gen_range(0, 1000) >= 990 {
                break;
            }
            if scrum_master.am_i_leader() {
                println!("[{}] soy el lider", id);

                // Simulo trabajo
                thread::sleep(Duration::from_millis(500));

                socket.set_read_timeout(Some(Duration::from_millis(1))).unwrap();
                while let Ok((_, from)) = socket.recv_from(&mut buf) {
                    socket.send_to("PONG".as_bytes(), from).unwrap();
                }
            } else {
                println!("[{}] soy una réplica", id);
                let leader_id = scrum_master.get_leader_id();

                thread::sleep(Duration::from_millis(1000));

                socket.send_to("PING".as_bytes(), id_to_dataaddr(leader_id)).unwrap();
                socket.set_read_timeout(Some(TIMEOUT)).unwrap();
                match socket.recv_from(&mut buf) {
                    Ok((_, _)) => {},
                    _ => {
                        scrum_master.find_new()
                    },
                }
            }
        }
        println!("[{}] sale de servicio", id);
        scrum_master.stop();
        thread::sleep(Duration::from_secs(20));
    }
}

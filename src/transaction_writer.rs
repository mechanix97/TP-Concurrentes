
use chrono::Local;
use std::fs;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct TransactionWriter {
    sender: Arc<Mutex<mpsc::Sender<String>>>,
    writter: Arc<Writter>,
    last_line: Arc<Mutex<Option<String>>>
}

impl TransactionWriter {
    pub fn new(file_name: String) -> TransactionWriter {
        let (sender, receiver) = mpsc::channel();
        let writter = Writter::new(Arc::new(Mutex::new(receiver)), file_name);
        TransactionWriter {
            sender: Arc::new(Mutex::new(sender)),
            writter: Arc::new(writter),
            last_line: Arc::new(Mutex::new(None)),
        }
    }

    pub fn log(&mut self, line: String) {
        if line.chars().last().unwrap() == '\n' {
            {
                *self.last_line.lock().unwrap() = Some(line.clone());
            }
            
            self.sender.lock().unwrap().send(line).unwrap();
        } else {
            {   
                *self.last_line.lock().unwrap() = Some(line.clone());
            }
            self.sender
                .lock()
                .unwrap()
                .send(format!("{}\n", line))
                .unwrap();
        }
    }

    pub fn last_line(&self) -> Option<String> {
        (*self.last_line.lock().unwrap()).clone()
    }
}

impl Clone for TransactionWriter {
    fn clone(&self) -> TransactionWriter {
        TransactionWriter {
            sender: self.sender.clone(),
            writter: self.writter.clone(),
            last_line: self.last_line.clone()
        }
    }
}

struct Writter {
    thread: Option<thread::JoinHandle<()>>,
}

impl Writter {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<String>>>, filename: String) -> Writter {
        let mut file = fs::File::create(&filename).unwrap();
        let thread = thread::spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(line) => {
                        file.write_all(
                            format!("{}: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), line)
                                .as_bytes(),
                        )
                        .unwrap();
                    }
                    Err(mpsc::RecvError) => break, //closed
                }
            }
        });

        Writter {
            thread: Some(thread),
        }
    }
}

impl Drop for Writter {
    fn drop(self: &mut Writter) {
        self.thread.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{prelude::*, BufReader};
    use super::*;

    #[test]
    fn file_seek(){
        let mut commiter = TransactionWriter::new(r"transactions\commits.txt".to_string());
        let t = "{\"id\":10,\"bank_payment\":{\"id\":10,\"amount\":100},\"flight_reservation\":{\"id\":10,\"amount\":100},\"hotel_reservation\":{\"id\":10,\"amount\":100}}".to_string();

        for _ in 0..10{
            commiter.log(t.clone());
        }
        
        let ll = match commiter.last_line() {
            Some(l) => l,
            None => {return;}
        };

        let filename = r"transactions\input.txt";
        let file = File::open(filename).expect("Error: file not found!");
        let mut count = 0;
        
        let reader = BufReader::new(file);

        let mut i = 0;

        for line in reader.lines(){
            i = i+1;
            let l = line.unwrap();
            if ll.eq(&l){
                break
            }

            count = count + l.chars().count() + 1;
        }

        let offset_file = File::open(filename).expect("Error: file not found!");
        let mut offset_reader = BufReader::new(offset_file);
        offset_reader.seek_relative(count.try_into().unwrap()).unwrap();
        let mut buf = String::new();
        offset_reader.read_line(&mut buf).unwrap();
        println!("{}", buf);
        offset_reader.read_line(&mut buf).unwrap();
        println!("{}", buf);

    }
}
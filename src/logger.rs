use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct Logger {
    sender: Arc<Mutex<mpsc::Sender<String>>>,
    writter: Arc<Writter>,
}

impl Logger {
    pub fn new(file_name: String) -> Logger {
        let (sender, receiver) = mpsc::channel();
        let writter = Writter::new(
            Arc::new(Mutex::new(receiver)),
            file_name,
        );
        Logger {
            sender: Arc::new(Mutex::new(sender)),
            writter: Arc::new(writter),
        }
    }

    pub fn log(&self, line: String) {
        self.sender.lock().unwrap().send(line).unwrap();
    }
}

impl Clone for Logger{
    fn clone(&self) -> Logger {
        Logger { sender: self.sender.clone(), writter: self.writter.clone() }
    }
}

pub struct Writter {
    filename: String,
    thread: Option<thread::JoinHandle<()>>,
}

impl Writter {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<String>>>, filename: String) -> Writter {
        let mut file = fs::File::create(&filename).unwrap();
        let thread = thread::spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(line) => file.write_all(line.as_bytes()).unwrap(),
                    Err(mpsc::RecvError) => break, //closed
                }
            }
        });

        Writter {
            filename: filename,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_thread_log() {
        fs::remove_file("test/multiple_thread_log.txt").unwrap();

        let (sender, receiver) = mpsc::channel();
        let writter = Writter::new(
            Arc::new(Mutex::new(receiver)),
            "test/multiple_thread_log.txt".to_string(),
        );

        let mut workers = Vec::new();

        let arc_sender = Arc::new(Mutex::new(sender));
        for i in 0..100 {
            let s = arc_sender.clone();
            let t = thread::spawn(move || {
                s.lock().unwrap().send(format!("Test {} \n", i)).unwrap();
            });
            workers.push(t);
        }

        for t in workers {
            t.join();
        }
        assert!(Path::new("test/multiple_thread_log.txt").exists());
    }
}

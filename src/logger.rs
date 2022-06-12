use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
/*
pub struct Logger{

}

impl Logger{
    pub fn new(file_name: String) -> Logger {

    }
}*/

pub struct Writter{
    filename: String,
    thread: Option<thread::JoinHandle<()>>,
}

impl Writter {

    fn new( receiver: Arc<Mutex<mpsc::Receiver<String>>>, filename: String) -> Writter {
        let mut file = File::create(&filename).unwrap();
        let thread = thread::spawn(move || {            
            loop {
                let line = receiver.lock().unwrap().recv().unwrap();
                file.write_all(line.as_bytes()).unwrap();
            }  
        });

        Writter {
            filename: filename,
            thread: Some(thread),
        }
    }
}
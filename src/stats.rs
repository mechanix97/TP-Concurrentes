use actix::clock::Instant;

use std::time::{Duration};

pub struct Stats{
    runs: Vec<Duration>,
    now: Instant,
    total_runs: u32,
    correct_runs: u32,
    failed_runs: u32
}

impl Stats{
    pub fn new() -> Stats{
        Stats { runs: Vec::new(), now: Instant::now(),total_runs:0, correct_runs: 0, failed_runs: 0 }
    }

    pub fn start_transaction(&mut self){
        self.now = Instant::now();
    }

    pub fn end_transaction(&mut self, st: bool){
        self.runs.push(self.now.elapsed());
        self.total_runs = self.total_runs + 1;
        match st {
            true => self.correct_runs = self.correct_runs + 1,
            false => self.failed_runs = self.failed_runs + 1,
        }
    }

    pub fn stop(&self){
        let mut total = Duration::new(0, 0);
        for d in &self.runs{
            total = total + *d;
        }
        let avg = total / self.total_runs;
        println!("Se ejcutaron {} transacciones en {} ms", self.total_runs, total.as_millis());
        println!("\tCorrectas: {}", self.correct_runs);
        println!("\tIncorrectas: {}", self.failed_runs);
        println!("\tTiempo medio de transaccion {} ms", avg.as_millis() );
    }

    
}
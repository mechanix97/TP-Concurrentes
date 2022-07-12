use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub use crate::logger::*;
pub use crate::transaction_writer::*;

pub fn exec(logger: Logger, mut commiter: TransactionWriter, remote_host: String, remote_port:String, transaction: String, leader_alive: Arc<AtomicBool>){
    logger.log(format!(
        "received Commit from {}:{}",
        remote_host, remote_port
    ));
    leader_alive.store(true, Ordering::Relaxed);
    commiter.log(transaction.clone());
}
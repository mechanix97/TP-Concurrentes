pub use crate::logger::*;
pub use crate::transaction_writer::*;

pub fn exec(logger: Logger, mut commiter: TransactionWriter, remote_host: String, remote_port:String, transaction: String){
    logger.log(format!(
        "received Commit from {}:{}",
        remote_host, remote_port
    ));
    //la.store(true, Ordering::Relaxed);
    commiter.log(transaction.clone());
}
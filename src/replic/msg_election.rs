use std::{io::Write, net::TcpStream};

pub use crate::commons::DistMsg;
pub use crate::logger::*;

pub fn exec(mut writer: TcpStream, id: u32, hostname: String, port: String, logger: Logger) {
    logger.log(format!(
        "received Election from {}:{}",
        hostname, port
    ));

    writer.write(
        &DistMsg::Election { id: id }.to_string().as_bytes())
        .unwrap();
}

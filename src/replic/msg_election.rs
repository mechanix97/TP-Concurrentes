use std::{io::Write, net::TcpStream};

pub use crate::commons::DistMsg;

pub fn exec(mut writer: TcpStream, id: u32) {
    writer.write(
        &DistMsg::Election { id: id }.to_string().as_bytes())
        .unwrap();
}

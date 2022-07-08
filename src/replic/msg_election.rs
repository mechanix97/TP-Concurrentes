use std::{net::TcpStream, io::Write};

pub use crate::commons::DistMsg;


pub fn exec(mut writer: TcpStream, id: u32) {
    writer
    .write(
        &(serde_json::to_string(&DistMsg::Election {
            id: id,
        })
        .unwrap()
            + "\n")
            .as_bytes(),
    )
    .unwrap();
}

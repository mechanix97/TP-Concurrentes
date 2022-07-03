use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Msg {
    Payment { id: i32, amount: i32 },
    Reversal { id: i32 },

    Quit,
}

pub fn deserialize(serialized: String) -> Result<Msg, serde_json::Error> {
    serde_json::from_str(&serialized)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DistMsg {
    Discover {
        id: u32,
        hostname: String,
        port: String,
    },
    NewReplic {
        id: u32,
        hostname: String,
        port: String,
    },
    Election{
        id: u32
    },
    Leader{
        id: u32
    },
    Ping,
    Pong,
}

pub fn deserialize_dist(serialized: String) -> Result<DistMsg, serde_json::Error> {
    serde_json::from_str(&serialized)
}

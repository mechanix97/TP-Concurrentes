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


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;


    #[test]

    fn generate_input_file() {
        let mut file = File::create("input.txt").unwrap();
        for _ in 0..100000{
            file.write_all(&(serde_json::to_string(&Msg::Payment {id: 1, amount: 100}) 
            .unwrap()
                + "\n").as_bytes() ).unwrap();
        }
    }
}
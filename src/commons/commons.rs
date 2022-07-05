use serde::{Deserialize, Serialize};

pub fn mock_response(len: usize, value: usize) -> usize {
    let mut length = len;
    if value < 4 {
        length = 0;
    }
    length
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment{pub id: i32, pub amount: f32}

pub fn deserialize(serialized: String) -> Result<Payment, serde_json::Error> {
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
        for i in 0..100000{
            file.write_all(&(serde_json::to_string(&Payment {id: i/3, amount: 100.0}) 
            .unwrap()
                + "\n").as_bytes() ).unwrap();
        }
    }
}
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Msg {
    Payment { id: i32, amount: i32 },
    Reversal { id: i32 },
    Ack,
    Nack,
    Quit,
}

pub fn deserialize(serialized: String) -> Result<Msg, serde_json::Error> {
    serde_json::from_str(&serialized)
}

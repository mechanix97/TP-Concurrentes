use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment{pub id: i32, pub amount: f32}

pub fn deserialize(serialized: String) -> Result<Payment, serde_json::Error> {
    serde_json::from_str(&serialized)
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment{
    id: i32,
    amount: f32,   
}

impl Payment {
    pub fn new(&self, line: &str) -> Payment {
        serde_json::from_str(line).unwrap() //revisar
    }
}
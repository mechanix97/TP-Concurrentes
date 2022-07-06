use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment{
    pub id: i32,
    pub amount: f32,   
}

impl Payment {
    pub fn new(&self, line: &str) -> Payment {
        serde_json::from_str(line).unwrap() //revisar
    }
}
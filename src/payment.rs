use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payment{
    kind: PaymentKind,
    amount: f32,   
}

enum PaymentKind{
    bank,
    hotel,
    airline,
}

impl Payment {
    pub fn new(&self, line: &str) -> Payment {
        serde_json::from_str(line).unwrap() //revisar
    }
}
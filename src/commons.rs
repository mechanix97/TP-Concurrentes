use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BankPayment {
    id: u32,
    amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FlightReservation {
    id: u32,
    amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HotelReservation {
    id: u32,
    amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Transaction {
    id: u32,
    bank_payment: BankPayment,
    flight_reservation: FlightReservation,
    hotel_reservation: HotelReservation,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ExternalMsg {
    Prepare {transaction: Transaction},
    NACK {id: u32},
    ACK {id: u32},
    Stop,
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
    Election {
        id: u32,
    },
    NewLeader {
        id: u32,
    },
    Commit {
        transaction: String,
    },
    Rollback {
        transaction: String,
    },
    Ping,
    Pong,
    Shutdown {
        hostname: String,
        port: String,
        shutdown: bool,
    },
}

impl DistMsg {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap() + "\n"
    }
}

impl ExternalMsg {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap() + "\n"
    }
}

impl Transaction {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap() + "\n"
    }

    #[allow(dead_code)]
    pub fn get_id(&self) -> u32 {
        self.id
    }
}

#[allow(dead_code)]
pub fn deserialize_transaction(serialized: String) -> Result<Transaction, serde_json::Error> {
    serde_json::from_str(&serialized)
}

#[allow(dead_code)]
pub fn deserialize_ext(serialized: String) -> Result<ExternalMsg, serde_json::Error> {
    serde_json::from_str(&serialized)
}

#[allow(dead_code)]
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
        for i in 0..100000 {
            let t = Transaction {
                id: i,
                bank_payment: BankPayment { id: i, amount: 100 },
                flight_reservation: FlightReservation { id: i, amount: 100 },
                hotel_reservation: HotelReservation { id: i, amount: 100 },
            };
            file.write_all(&(serde_json::to_string(&t).unwrap() + "\n").as_bytes())
                .unwrap();
        }
    }
}


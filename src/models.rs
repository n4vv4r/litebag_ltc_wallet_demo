use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub hash: String,
    pub amount: i64, 
    pub confirmations: i32,
    pub received: String, 
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxInput {
    pub addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxOutput {
    pub addresses: Vec<String>,
    pub value: i64,
}
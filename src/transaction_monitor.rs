use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use crate::models::Transaction;
use crate::database::save_transaction;

const BLOCKCYPHER_API_KEY: &str = "api de blockcypher"; 

#[derive(Serialize, Deserialize, Debug)]
struct BlockCypherResponse {
    txs: Vec<Transaction>, 
}

pub async fn get_transactions(address: &str) -> Result<Vec<Transaction>, Error> {
    let url = format!(
        "https://api.blockcypher.com/v1/ltc/main/addrs/{}/full?limit=50&token={}",
        address, BLOCKCYPHER_API_KEY
    );
    let response = reqwest::get(&url).await?.json::<BlockCypherResponse>().await?;
    Ok(response.txs) 
}

pub async fn monitor_transactions(address: &str, path: &std::path::PathBuf) {
    let mut seen_txs = HashSet::new(); 

    loop {
        println!("Monitoreando transacciones para la dirección: {}", address);

        match get_transactions(address).await {
            Ok(transactions) => {
                if transactions.is_empty() {
                    println!("No hay transacciones nuevas.");
                } else {
                    for tx in transactions {
                        if !seen_txs.contains(&tx.hash) && tx.confirmations > 0 {
                            println!("Nueva transacción recibida: {:?}", tx);
                            seen_txs.insert(tx.hash.clone());

                            if let Err(e) = save_transaction(path, &tx) {
                                eprintln!("Error al guardar la transacción: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error al obtener transacciones: {}", e);
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}

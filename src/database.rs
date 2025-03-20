use rusqlite::{Connection, params};
use std::path::PathBuf;
use crate::models::Transaction;

pub fn get_latest_address(path: &PathBuf) -> Result<String, String> {
    let conn = Connection::open(path).map_err(|e| format!("Fallado al abrir wallet: {}", e))?;

    let mut stmt = conn.prepare("SELECT address FROM addresses ORDER BY id DESC LIMIT 1")
        .map_err(|e| format!("Fallado al hacer query a direcciones: {}", e))?;

    let address: String = stmt.query_row([], |row| row.get(0))
        .map_err(|e| format!("Fallado al leer dirección: {}", e))?;

    Ok(address)
}

pub fn save_transaction(path: &PathBuf, tx: &Transaction) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| format!("Fallado al abrir wallet: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY,
            hash TEXT NOT NULL,
            amount INTEGER NOT NULL,
            confirmations INTEGER NOT NULL,
            received TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("Fallado al crear tabla: {}", e))?;

    conn.execute(
        "INSERT INTO transactions (hash, amount, confirmations, received) VALUES (?1, ?2, ?3, ?4)",
        params![tx.hash, tx.amount, tx.confirmations, tx.received],
    ).map_err(|e| format!("Fallado al insertar transacción: {}", e))?;

    Ok(())
}

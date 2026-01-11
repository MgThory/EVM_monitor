use crate::config::MonitorContract;
use alloy::primitives::Address;
use rusqlite::{types::Type, Connection, Error as SqlError};
use std::error::Error;
use std::str::FromStr;

const DB_PATH: &str = "myminitor.db";

/// Open (or create) the SQLite database and ensure the monitor table exists.
pub fn init_db() -> Result<Connection, Box<dyn Error>> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS monitor_contracts (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            address     TEXT NOT NULL,
            chain_id    INTEGER,
            start_block INTEGER,
            is_active   INTEGER NOT NULL DEFAULT 1
        );
        ",
        [],
    )?;
    Ok(conn)
}

/// Load all active monitor contracts from the database.
pub fn load_monitor_contracts(conn: &Connection) -> Result<Vec<MonitorContract>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT name, address, chain_id, start_block, is_active
        FROM monitor_contracts
        WHERE is_active = 1
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        let addr_str: String = row.get(1)?;
        let address = Address::from_str(&addr_str).map_err(|_| {
            SqlError::InvalidColumnType(1, "address".into(), Type::Text)
        })?;

        Ok(MonitorContract {
            name: row.get(0)?,
            address,
            chain_id: row.get(2)?,
            start_block: row.get(3)?,
            is_active: row.get(4)?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }

    Ok(out)
}

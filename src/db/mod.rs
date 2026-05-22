use crate::config::MonitorContract;
use crate::log;
use alloy::primitives::Address;
use alloy::rlp::bytes::buf::Chain;
use mysql::{prelude::Queryable, Pool, PooledConn};
use std::error::Error;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

pub struct RuntimeState {
    pub chain_id: u64,
    pub last_block: u64,
    pub last_event_ts: u64,
    pub updated_at: u64,
}

fn connect() -> Result<PooledConn, Box<dyn Error>> {
    let url = std::env::var("DB_PATH")?;
    let pool = Pool::new(url.as_str())?;
    Ok(pool.get_conn()?)
}

/// Load all active monitor contracts from the database.
pub fn load_monitor_contracts() -> Result<Vec<MonitorContract>, Box<dyn Error>> {
    let mut conn = connect()?;
    let rows: Vec<(String, String, Option<u64>, Option<u64>, bool)> = conn.query(
        "SELECT name, address, chain_id, start_block, is_active FROM monitor_contracts WHERE is_active = 1",
    )?;
    let mut out = Vec::new();
    for (name, addr, chain_id, start_block, is_active) in rows {
        let address = Address::from_str(&addr)?;
        out.push(MonitorContract { name, address, chain_id, start_block, is_active });
    }
    Ok(out)
}

pub fn save_runtime_state(state: &RuntimeState) -> Result<(), Box<dyn Error>> {
    let mut conn = connect()?;
    conn.exec_drop(
        "INSERT INTO runtime_state (id, chain_id, last_block, last_event_ts, updated_at)
        VALUES (1, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            chain_id=VALUES(chain_id),
            last_block=VALUES(last_block),
            last_event_ts=VALUES(last_event_ts),
            updated_at=VALUES(updated_at)",
        (state.chain_id, state.last_block, state.last_event_ts, state.updated_at),
    )?;
    Ok(())
}

pub fn load_runtime_state() -> Result<Option<RuntimeState>, Box<dyn Error>> {
    let mut conn = connect()?;
    let row: Option<(Option<u64>, Option<u64>, Option<u64>, Option<u64>)> = conn.query_first(
        "SELECT chain_id, last_block, last_event_ts, updated_at FROM runtime_state WHERE id=1",
    )?;
    Ok(match row {
        Some((Some(chain_id), Some(last_block), Some(last_event_ts), Some(updated_at))) => {
            Some(RuntimeState {chain_id, last_block, last_event_ts, updated_at})
        }
        _ => None,
    })
}

/// Load the last processed block (checkpoint).
pub fn load_checkpoint() -> Result<Option<u64>, Box<dyn Error>> {
    let mut conn = connect()?;
    let row: Option<Option<u64>> =
        conn.query_first("SELECT last_block FROM runtime_state WHERE id=1")?;
    Ok(row.flatten())
}

/// Save the last processed block (checkpoint).
pub fn save_checkpoint(block: u64) -> Result<(), Box<dyn Error>> {
    let mut conn = connect()?;
    conn.exec_drop(
        "INSERT INTO runtime_state (id, last_block) VALUES (1, ?) \
         ON DUPLICATE KEY UPDATE last_block=VALUES(last_block)",
        (block,),
    )?;
    Ok(())
}

pub fn add_monitor_contract(
    name: &str,
    address: &str,
    chain_id: Option<u64>,
    start_block: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    let mut conn = connect()?;
    conn.exec_drop(
        "INSERT INTO monitor_contracts (name, address, chain_id, start_block, is_active) VALUES (?, ?, ?, ?, 1)",
        (name, address, chain_id, start_block),
    )?;
    Ok(())
}

pub fn delete_monitor_contract(id: u64) -> Result<(), Box<dyn Error>> {
    let mut conn = connect()?;
    conn.exec_drop("DELETE FROM monitor_contracts WHERE id = ?", (id,))?;
    Ok(())
}

//将数据库中的被检测合约列举出来
pub fn list_monitor_contracts() -> Result<Vec<(u64, String, String)>, Box<dyn Error>> {
    let mut conn = connect()?;
    let rows: Vec<(u64, String, String)> = 
        conn.query("SELECT id, name, address FROM monitor_contracts ORDER BY id ASC")?;
    Ok(rows)
}

/// Load active contracts; fallback to env vars if DB is empty. Applies checkpoint to start_block when missing.
pub fn load_contracts_with_env_fallback() -> Result<Vec<MonitorContract>, Box<dyn Error>> {
    loop {
        let mut contracts = load_monitor_contracts()?;
        let checkpoint = load_runtime_state()?;

        if contracts.is_empty() {
            log::info("No active monitor_contracts found; waiting for DB data...");
            thread::sleep(Duration::from_secs(5));
            continue;
        }

        if let Some(cp) = checkpoint {
            for c in &mut contracts {
                if c.start_block.is_none() && c.chain_id == Some(cp.chain_id) {
                    c.start_block = Some(cp.last_block + 1);
                }
            }
        }

        return Ok(contracts);
    }
}

use alloy::primitives::Address;
use myfuckingreallymonitor::config::{Event, MonitorContract};
use myfuckingreallymonitor::connector::BlockchainConnector;
use myfuckingreallymonitor::db;
use myfuckingreallymonitor::processor::{prase::process_log, Catcher};
use std::error::Error;
use std::str::FromStr;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, myfuckingreallymonitor!");

    let connector = BlockchainConnector::build()?;

    match connector.get_block_number().await {
        Ok(num) => println!("Current block number: {}", num),
        Err(e) => println!("Error fetching block number: {}", e),
    }

    let db = db::init_db()?;
    let mut contracts = db::load_monitor_contracts(&db)?;

    if contracts.is_empty() {
        println!("DB monitor_contracts table is empty; falling back to env MONITOR_CONTRACT_ADDRESS.");
        let contract_addr = match std::env::var("MONITOR_CONTRACT_ADDRESS") {
            Ok(addr) => match Address::from_str(&addr) {
                Ok(parsed) => parsed,
                Err(_) => {
                    eprintln!("MONITOR_CONTRACT_ADDRESS is invalid; expected hex address (0x...).");
                    return Ok(());
                }
            },
            Err(_) => {
                println!("MONITOR_CONTRACT_ADDRESS not set; no contracts to monitor.");
                return Ok(());
            }
        };

        let contract_name = std::env::var("MONITOR_CONTRACT_NAME")
            .unwrap_or_else(|_| "MonitoredContract".to_string());

        contracts.push(MonitorContract {
            address: contract_addr,
            name: contract_name,
            chain_id: None,
            start_block: None,
            is_active: true,
        });
    }

    let (tx, mut rx) = mpsc::channel::<Event>(100);
    let catcher = Catcher::new(connector, contracts);
    catcher.run(tx).await?;

    while let Some(event) = rx.recv().await {
        process_log(event);
    }

    Ok(())
}

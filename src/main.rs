use chrono::format;
use evm_monitor::connector::BlockchainConnector;
use evm_monitor::db;
use evm_monitor::log;
use evm_monitor::processor::run_pipeline;
use std::error::Error;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::init()?;
    log::info("Hello, EVM_Monitor!");

    let connector = BlockchainConnector::build()?;

    // 异步等待DB出现可用地址
    let contracts = tokio::task::spawn_blocking(|| {
        db::load_contracts_with_env_fallback().map_err(|e| e.to_string())
    })
    .await??;


    log_startup(&contracts);
    run_pipeline(connector, contracts).await?;
    Ok(())
}

fn log_startup(contracts: &[evm_monitor::config::MonitorContract]) {
    let mut entries = Vec::new();
    for c in contracts {
        let from = c.start_block.map_or_else(|| "latest".to_string(), |b| b.to_string());
        entries.push(format!("{} (from={})", c.name, from));
    }
    log::info(&format!("startup, monitoring: {}", entries.join("; ")));
    log::info(&format!("Load {} contracts", contracts.len()));
    for c in contracts {
        log::info(&format!(
            "contract name: {}, chain_id: {:?}, start_block: {:?}",
            c.name, c.chain_id, c.start_block
        ));
    }
}

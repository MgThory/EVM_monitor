use crate::config::{Event, MonitorContract};
use crate::connector::BlockchainConnector;

use alloy::providers::Provider;
use alloy::rpc::types::eth::Filter;
use alloy::rpc::types::BlockNumberOrTag;
use futures_util::StreamExt;
use tokio::sync::mpsc;

pub struct Catcher {
    contracts: Vec<MonitorContract>,
    connector: BlockchainConnector,
}

impl Catcher {
    pub fn new(connector: BlockchainConnector, contracts: Vec<MonitorContract>) -> Self {
        Self { contracts, connector }
    }

    pub async fn run(&self, sender: mpsc::Sender<Event>) -> Result<(), String> {
        println!("Catcher Starting");

        for contract in &self.contracts {
            if !contract.is_active {
                continue;
            }

            let my_connector = self.connector.clone();

            let tx = sender.clone();
            let c_addr = contract.address;
            let c_name = contract.name.clone();
            let start_block = contract.start_block;
            let c_chain_id = contract.chain_id;

            tokio::spawn(async move {
                println!("Start watching : {} Address: {}", c_name, c_addr);

                let mut filter = Filter::new().address(c_addr);
                filter = match start_block {
                    Some(from) => filter.from_block(from),
                    None => filter.from_block(BlockNumberOrTag::Latest),
                };

                match my_connector.provider.watch_logs(&filter).await {
                    Ok(sub) => {
                        let mut stream = sub.into_stream();
                        while let Some(logs) = stream.next().await {
                            for log in logs {
                                let event = Event {
                                    contract_name: c_name.clone(),
                                    chain_id: c_chain_id,
                                    tx_hash: log.transaction_hash,
                                    log,
                                };
                                if tx.send(event).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            });
        }

        Ok(())
    }
}

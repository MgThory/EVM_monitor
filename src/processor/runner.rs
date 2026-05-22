use crate::config::{Event, MonitorContract};
use crate::connector::BlockchainConnector;
use crate::db;
use crate::processor::{catcher::Catcher, format_alert_message, parse_event};
use std::error::Error;
use tokio::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Orchestrates catching, parsing, and alerting.
pub async fn run_pipeline(
    connector: BlockchainConnector,
    contracts: Vec<MonitorContract>,
) -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel::<Event>(100);
    let catcher = Catcher::new(connector, contracts);
    let mut lastest_state : Option<db::RuntimeState> = None;
    catcher.run(tx).await?;



    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if let Some(state) = &lastest_state {
                    if let Err(e) = db::save_runtime_state(state) {
                        eprint!("Failed to save runtime state on shutdown: {}", e);
                    }
                }
                break;
            }


            maybe_event = rx.recv() => {
                let Some(event) = maybe_event else { break; };

                if let (Some(chain_id), Some(block_number)) = (event.chain_id, event.log.block_number) {
                    let event_ts = event.log.block_timestamp.unwrap_or_else(|| {
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    });

                    let now_ts = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    lastest_state = Some(db::RuntimeState {
                        chain_id,
                        last_block: block_number,
                        last_event_ts: event_ts,
                        updated_at: now_ts,
                    });
                }
                
                let parsed = parse_event(event);
                let message = format_alert_message(&parsed);
                println!("{}", message);

            }
        }
    }
    
    Ok(())
    }


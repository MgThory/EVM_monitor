use alloy::primitives::{Address, TxHash};
use alloy::rpc::types::eth::Log;

pub struct AppConfig {
    
}

// 监控配置
#[derive(Clone, Debug)]
pub struct MonitorContract {
    pub address: Address,
    pub name: String,
    pub chain_id: Option<u64>,
    pub start_block: Option<u64>,
    pub is_active: bool,
}


// Catcher 抓到的东西
#[derive(Clone, Debug)]
pub struct Event {
    pub contract_name: String,
    pub tx_hash: Option<TxHash>,
    pub log: Log,
}


impl AppConfig {
    pub fn new() -> Self {


        Self{}
    }

}

use alloy::primitives::{Address, TxHash};
use alloy::rpc::types::eth::Log;

// 全局配置常量
pub const DB_PATH: &str = "myminitor.db";
pub const DEFAULT_DECIMALS: u32 = 6;
pub const ALERT_WARNING_USDC: u128 = 1_000;
pub const ALERT_EMERGENCY_USDC: u128 = 10_000;

/// 将链简称转换为链 ID（仅针对 EVM 链）。
/// 示例：ETH -> 1, BSC -> 56, POLYGON -> 137, ARB -> 42161, OPT -> 10, BASE -> 8453, SEPOLIA -> 11155111, BSC_TEST -> 97
pub fn resolve_chain_id(alias: &str) -> Option<u64> {
    match alias.to_ascii_uppercase().as_str() {
        "ETH" | "MAINNET" => Some(1),
        "SEPOLIA" => Some(11155111),
        "BSC" => Some(56),
        "BSC_TEST" | "BSC-TEST" | "BSC_TESTNET" => Some(97),
        "POLYGON" | "MATIC" => Some(137),
        "ARB" | "ARBITRUM" => Some(42161),
        "OPT" | "OPTIMISM" => Some(10),
        "BASE" => Some(8453),
        _ => None,
    }
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
    pub chain_id: Option<u64>,
}

use alloy::primitives::utils::format_units;
use alloy::primitives::{Address, B256, TxHash, U256};
use alloy::sol;
use alloy::sol_types::SolEvent;
use chrono::Local;

use crate::config::{Event, ALERT_EMERGENCY_USDC, ALERT_WARNING_USDC, DEFAULT_DECIMALS};

// 维护 ERC20 事件定义
sol! {
    #[derive(Debug)]
    event Transfer(address indexed from, address indexed to, uint256 value);

    #[derive(Debug)]
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

#[derive(Debug, Clone, Copy)]
pub enum AlertLevel {
    Normal,
    Warning,
    Emergency,
}

#[derive(Debug, Clone)]
pub enum ParsedEventKind {
    Transfer {
        from: Address,
        to: Address,
        value: U256,
        value_formatted: String,
        level: AlertLevel,
    },
    Approval {
        owner: Address,
        spender: Address,
        value: U256,
        value_formatted: String,
    },
    Unknown {
        topic0: Option<B256>,
    },
}

#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub contract_name: String,
    pub tx_hash: Option<TxHash>,
    pub kind: ParsedEventKind,
}

fn now_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 主入口：解析一条链上事件
pub fn parse_event(event: Event) -> ParsedEvent {
    let topic0 = event.log.topic0().copied();

    // 优先尝试 Transfer
    if let Ok(decoded) = Transfer::decode_log(&event.log.inner) {
        let level = classify_transfer(decoded.value);
        let formatted =
            format_units(decoded.value, DEFAULT_DECIMALS as u8).unwrap_or_else(|_| "0".into());
        return ParsedEvent {
            contract_name: event.contract_name,
            tx_hash: event.tx_hash,
            kind: ParsedEventKind::Transfer {
                from: decoded.from,
                to: decoded.to,
                value: decoded.value,
                value_formatted: formatted,
                level,
            },
        };
    }

    // 再尝试 Approval
    if let Ok(decoded) = Approval::decode_log(&event.log.inner) {
        let formatted =
            format_units(decoded.value, DEFAULT_DECIMALS as u8).unwrap_or_else(|_| "0".into());
        return ParsedEvent {
            contract_name: event.contract_name,
            tx_hash: event.tx_hash,
            kind: ParsedEventKind::Approval {
                owner: decoded.owner,
                spender: decoded.spender,
                value: decoded.value,
                value_formatted: formatted,
            },
        };
    }

    ParsedEvent {
        contract_name: event.contract_name,
        tx_hash: event.tx_hash,
        kind: ParsedEventKind::Unknown { topic0 },
    }
}

/// 根据金额做告警分级（默认 6 位小数的 ERC20）
fn classify_transfer(value: U256) -> AlertLevel {
    let unit = U256::from(10u128.pow(DEFAULT_DECIMALS.into()));
    let warning = U256::from(ALERT_WARNING_USDC) * unit;
    let emergency = U256::from(ALERT_EMERGENCY_USDC) * unit;

    if value >= emergency {
        AlertLevel::Emergency
    } else if value >= warning {
        AlertLevel::Warning
    } else {
        AlertLevel::Normal
    }
}

/// 将解析结果组装成可读的告警文本（供 stdout/TG 复用）
pub fn format_alert_message(event: &ParsedEvent) -> String {
    let now = now_time();

    match &event.kind {
        ParsedEventKind::Transfer { from, to, value_formatted, level, .. } => {
            format!(
                "[{}] [{}] [{:?}] Transfer From: {:#x} To: {:#x} Amount: {}",
                now,
                event.contract_name,
                level,
                from,
                to,
                value_formatted
            )
        }
        ParsedEventKind::Approval { owner, spender, value_formatted, .. } => format!(
            "[{}] [{}][Approval] Owner: {:#x} Spender: {:#x} Value: {}",
            now,event.contract_name, owner, spender, value_formatted
        ),
        ParsedEventKind::Unknown { topic0 } => format!(
            "[{}] [{}][Unknown] topic0: {:?} Tx: {:?}",
            now,event.contract_name, topic0, event.tx_hash
        ),
    }
}

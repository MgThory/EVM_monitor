use alloy::dyn_abi::abi::decode;
use alloy::sol;
use alloy::sol_types::SolEvent; // 必须引入这个 trait 才能调用 decode_log
use alloy::primitives::utils::format_units; // 用于把 wei 转成 eth/usdt

use crate::config::Config::Event;

sol! {
    #[derive(Debug)]
    event Transfer(address indexed from, address indexed to, uint256 value);

    #[derive(Debug)]
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

pub fn process_log(event: Event) {
    let log = event.log;

    // try_decode_log 会尝试检查 topic0 是否匹配 Transfer 的签名
    // 如果匹配，它会解析 data 部分
    match Transfer::decode_log(&log.inner) {
        Ok(decoded_data) => {
            let from = decoded_data.from;
            let to = decoded_data.to;
            let value = decoded_data.value;

            let fmt_amount = format_units(value, 6).unwrap_or_default();

            println!("[Event: Transfer Contract: {} Tx: {:?}]", event.contract_name, event.tx_hash);
            println!("From: {:?}", from);
            println!("To: {:?}", to);
            println!("Value: {}",fmt_amount);

            //TODO：这里写入数据库，或者发送tlelgram报警
        },
        Err(_) => {
            println!("[{}], Unknow Type Log (Topic: {:?})",
                event.contract_name, log.topic0());
        }
    }
}

use alloy::primitives::Address;
use crate::db;
use std::error::Error;
use std::str::FromStr;


fn usage() {
    eprintln!("Usage:");
    eprintln!("morning --addaddress  <name> <address>");
    eprintln!("morning --listaddresses");
    eprintln!("morning --deladdress <index>");
}


/// 处理地址相关的 CLI 命令
pub fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        usage();
        return Ok(());
    }

    //增添地址
    if args.len() == 3 && args[0] == "--addaddress" {
        let name = args[1].trim();
        if name.is_empty() {
            return Err("name cannot be empty".into());
        }
        let parsed = Address::from_str(&args[2]).map_err(|_| "Invalid EVM address")?;
        let address = format!("{:#x}", parsed);

        db::add_monitor_contract(&name, &address, None, None)?;
        println!("added: id(auto), name={}, address={}", name, address);
        return Ok(());
    }

    //列出地址
    if args.len() == 1 && args[0] == "--listaddresses" {
        let rows = db::list_monitor_contracts()?;
        if rows.is_empty() {
            println!("No monitored contracts");
            return Ok(());
        }
        println!("{:<8} {:<8} {:<20} {}", "index", "id", "name", "address");
        for (i, (id, name, address)) in rows.iter().enumerate() {
            println!("{:<8} {:<8} {:<20} {}", i + 1, id, name, address);
        }
        return Ok(());
    }

    //删除地址
    if args.len() == 2 && args[0] == "--deladdress" {
        let index: usize = args[1].parse().map_err(|_| "index must be positive integer")?;
        if index == 0 {
            return Err("index starts from 1".into());
        }
        let rows = db::list_monitor_contracts()?;
        if index > rows.len() {
            return Err(format!("index out of range: 1-{}", rows.len()).into());
        }
        let (id, name, address) = &rows[index - 1];
        db::delete_monitor_contract(*id)?;
        println!("deleted: index={}, id={}, name={}, address={}", index, id, name, address);
        return Ok(());
    }

    Ok(())

}
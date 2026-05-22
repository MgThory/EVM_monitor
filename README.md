# EVM_Monitor 项目文档

## 1. 项目简介

`EVM_Monitor` 是一个基于 Rust 的 EVM 合约事件监控工具，核心能力是：

- 通过 RPC 订阅目标合约日志（`watch_logs`）。
- 解析 ERC20 常见事件（`Transfer` / `Approval`）。
- 按金额阈值进行告警分级（`Normal` / `Warning` / `Emergency`）。
- 将告警输出到控制台，或发送到 Telegram。
- 支持通过 Telegram 指令动态管理监控合约（增删查）。
- 在 MySQL 中保存运行断点（checkpoint）与监控合约列表。

## 2. 当前代码状态（2026-03-10）

在仓库根目录执行 `cargo check` 的结果：**未通过**。主要问题：

- 缺少依赖：`tracing`、`serde`、`serde_json`。
- `tracing-subscriber` 缺少 `chrono` 与 `env-filter` 特性，导致 `ChronoLocal` / `EnvFilter` 无法导入。

这意味着当前代码需要先补齐依赖与 feature，才能正常编译运行。

## 3. 技术栈

- 语言与异步：Rust 2024 + Tokio
- 链交互：`alloy`
- HTTP：`reqwest`（Telegram API）
- 数据库：`mysql`
- 日志：`tracing` 生态（当前 `Cargo.toml` 未完整配置）
- 配置加载：`dotenvy`

## 4. 项目结构

```text
src/
  main.rs                    # 入口：初始化日志、连接器、机器人、启动 pipeline
  lib.rs                     # 模块导出
  config/
    Config.rs                # 常量、链映射、核心数据结构
    mod.rs
  connector/
    ConnectToBlockchain.rs   # RPC provider 构建与基础链查询
    mod.rs
  db/
    mod.rs                   # MySQL 读写：监控合约、checkpoint
  processor/
    catcher.rs               # 日志订阅/抓取
    parser.rs                # 事件解码、告警分级、消息格式化
    runner.rs                # 抓取-解析-告警流水线编排
    mod.rs
  tg_bot/
    telegram.rs              # Telegram 发消息、轮询、命令处理
    mod.rs
  log/
    mod.rs                   # tracing 日志初始化
db.sql                       # MySQL 表结构初始化脚本
```

## 5. 核心运行流程

1. `main.rs` 初始化日志。
2. 构建链连接器（读取 `MAINNET_RPC_URL`）。
3. 尝试构建 TelegramBot（读取 `TELEGRAM_BOT_TOKEN`、`TELEGRAM_CHAT_ID`）。
4. 启动 Telegram 命令轮询协程。
5. 循环加载 DB 的 `monitor_contracts`，直到存在 `is_active=1` 的合约。
6. 启动 `run_pipeline`：
   - `Catcher` 为每个合约启动日志订阅任务；
   - 接收链上事件后解析；
   - 生成告警文案；
   - 发送到 Telegram 或打印到 stdout；
   - 保存最新区块号到 `runtime_state.last_block`。

## 6. 事件解析与告警策略

当前解析器支持：

- `Transfer(address from, address to, uint256 value)`
- `Approval(address owner, address spender, uint256 value)`

默认小数位：`DEFAULT_DECIMALS = 6`（按 USDC 口径）。

分级阈值（单位：USDC）：

- `WARNING >= 1,000`
- `EMERGENCY >= 10,000`

## 7. Telegram 功能

### 7.1 支持命令

- `/start`：发送内联菜单
- `/add <name> <address> <chain>`：新增监控地址
- `/delete <id>`：删除监控地址
- `/list`：查看所有监控地址

### 7.2 支持链别名

- `ETH` / `MAINNET` -> `1`
- `SEPOLIA` -> `11155111`
- `BSC` -> `56`
- `BSC_TEST` / `BSC-TEST` / `BSC_TESTNET` -> `97`
- `POLYGON` / `MATIC` -> `137`
- `ARB` / `ARBITRUM` -> `42161`
- `OPT` / `OPTIMISM` -> `10`
- `BASE` -> `8453`

## 8. 数据库设计（MySQL）

`db.sql` 提供了两个核心表：

### 8.1 `monitor_contracts`

- `id`：主键
- `name`：业务名称
- `address`：合约地址
- `chain_id`：链 ID
- `start_block`：订阅起始块（可空）
- `is_active`：是否启用

### 8.2 `runtime_state`

- `id`：固定为 `1`
- `last_block`：最近处理区块号（断点）

## 9. 环境变量

程序中实际读取的环境变量：

- `MAINNET_RPC_URL`：EVM RPC 地址
- `DB_PATH`：**MySQL 连接串**（变量名虽为 PATH，但代码中用于 `mysql::Pool::new`）
- `TELEGRAM_BOT_TOKEN`：Telegram Bot Token（可选）
- `TELEGRAM_CHAT_ID`：Telegram Chat ID（可选）

建议示例（请按实际值替换）：

```env
MAINNET_RPC_URL=https://...
DB_PATH=mysql://user:password@host:3306/database
TELEGRAM_BOT_TOKEN=123456:xxxx
TELEGRAM_CHAT_ID=123456789
```

## 10. 启动方式

1. 准备 MySQL 并执行 `db.sql`。
2. 设置 `.env` 中的必要环境变量。
3. 修复当前编译依赖问题（见第 2 节）。
4. 启动：

```bash
cargo run
```

## 11. 日志

- 运行日志写入 `log/system.log.*`（按天滚动）。
- 同时输出到 stdout。
- 支持 `RUST_LOG` 过滤级别（默认 `info`）。

## 12. 已知问题与风险

- 项目当前不可直接编译（依赖与 feature 未配齐）。
- `DB_PATH` 命名与用途不一致（实际是 MySQL URL，容易误导）。
- `main.rs` 与 `runner.rs` 都启动了 `run_command_loop`，存在重复轮询风险。
- 连接器仅使用 `MAINNET_RPC_URL`，但配置层支持多链 `chain_id`，多链能力尚未闭环。
- `rusqlite` 依赖和仓库中的 `myminitor.db` 文件目前未被主流程使用，存在历史遗留痕迹。

---

如果你希望，我可以下一步直接给这份文档补一版“可运行修复清单（按 commit 粒度）”，把依赖补齐并消除重复轮询问题。

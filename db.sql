-- 表：监控合约
CREATE TABLE IF NOT EXISTS monitor_contracts (
    id          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    address     VARCHAR(66) NOT NULL,
    chain_id    BIGINT UNSIGNED NULL,
    start_block BIGINT UNSIGNED NULL,
    is_active   TINYINT(1) NOT NULL DEFAULT 1,
    KEY idx_address (address)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 表：运行时断点（只存一行，id=1）
CREATE TABLE IF NOT EXISTS runtime_state (
    id              TINYINT UNSIGNED NOT NULL PRIMARY KEY,
    chain_id        BIGINT UNSIGNED NULL,
    last_block      BIGINT UNSIGNED NULL,
    last_event_ts   BIGINT UNSIGNED NULL,
    updated_at      BIGINT UNSIGNED NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


-- 可选：初始化断点行（忽略已存在）
INSERT INTO runtime_state (id, last_block)
VALUES (1, NULL)
ON DUPLICATE KEY UPDATE last_block = last_block;

-- 可选：示例合约
-- INSERT INTO monitor_contracts (name, address, chain_id, start_block, is_active)
-- VALUES ('USDC', '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', 1, 19000000, 1);

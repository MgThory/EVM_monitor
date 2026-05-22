use alloy::providers::{
    DynProvider,
    Provider,
    ProviderBuilder,
};
use alloy::network::Ethereum;
use std::error::Error;
use url::Url;
use dotenvy::dotenv;
use std::sync::Arc;

// 定义一个类型别名，让代码更简洁
type EthProvider = DynProvider<Ethereum>;

#[derive(Clone)]
//定义一个provider，以后对区块链的请求就从这里拿
pub struct BlockchainConnector {
    pub provider: Arc<EthProvider>,
}

impl BlockchainConnector {
    pub fn build() -> Result<BlockchainConnector, Box<dyn Error>> {
        dotenv().ok();
        let rpc_url = std::env::var("MAINNET_RPC_URL")?;
        let url = Url::parse(&rpc_url)?;

        let provider = ProviderBuilder::new()
            .connect_http(url)
            .erased();

        Ok(BlockchainConnector {
            provider: Arc::new(provider),
        })

    }


    pub async fn get_block_number(&self) -> Result<u64, Box<dyn Error>> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number)
    }

}



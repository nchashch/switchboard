use crate::api::{Balances, BlockCounts, Chain, Sidechain, SidechainClient};
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use serde_json::Value;

pub struct Switchboardd {
    client: SidechainClient,
}

impl Switchboardd {
    pub fn new(client: SidechainClient) -> Switchboardd {
        Switchboardd { client }
    }
}

#[rpc(server, client)]
pub trait SwitchboardRpc {
    #[method(name = "generate")]
    async fn generate(
        &self,
        number: usize,
        amount: u64,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error>;

    #[method(name = "getbalances")]
    async fn getbalances(&self) -> Result<Balances, jsonrpsee::core::Error>;

    #[method(name = "getblockcounts")]
    async fn getblockcounts(&self) -> Result<BlockCounts, jsonrpsee::core::Error>;

    #[method(name = "getnewaddress")]
    async fn getnewaddress(&self, chain: Chain) -> Result<String, jsonrpsee::core::Error>;

    #[method(name = "deposit")]
    async fn deposit(
        &self,
        sidechain: Sidechain,
        amount: u64,
        fee: u64,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error>;

    #[method(name = "withdraw")]
    async fn withdraw(
        &self,
        sidechain: Sidechain,
        amount: u64,
        fee: u64,
    ) -> Result<String, jsonrpsee::core::Error>;

    #[method(name = "main")]
    async fn main(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error>;

    #[method(name = "zcash")]
    async fn zcash(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error>;

    #[method(name = "activatesidechains")]
    async fn activatesidechains(&self) -> Result<(), jsonrpsee::core::Error>;
}

#[async_trait]
impl SwitchboardRpcServer for Switchboardd {
    async fn generate(
        &self,
        number: usize,
        amount: u64,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        self.client.generate(number, amount).await
    }

    async fn getbalances(&self) -> Result<Balances, jsonrpsee::core::Error> {
        Ok(self.client.get_balances().await?)
    }

    async fn getblockcounts(&self) -> Result<BlockCounts, jsonrpsee::core::Error> {
        Ok(self.client.get_block_counts().await?)
    }

    async fn getnewaddress(&self, chain: Chain) -> Result<String, jsonrpsee::core::Error> {
        self.client.get_new_address(chain).await
    }

    async fn deposit(
        &self,
        sidechain: Sidechain,
        amount: u64,
        fee: u64,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error> {
        self.client.deposit(sidechain, amount, fee).await
    }

    async fn withdraw(
        &self,
        sidechain: Sidechain,
        amount: u64,
        fee: u64,
    ) -> Result<String, jsonrpsee::core::Error> {
        self.client.withdraw(sidechain, amount, fee).await
    }

    async fn main(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error> {
        self.client.main_request(method, params).await
    }

    async fn zcash(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error> {
        self.client.zcash_request(method, params).await
    }

    async fn activatesidechains(&self) -> Result<(), jsonrpsee::core::Error> {
        self.client.activate_sidechains().await
    }
}

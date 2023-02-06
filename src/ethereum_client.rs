use jsonrpsee::proc_macros::rpc;
use std::collections::HashMap;
use web3::types::{H256, U256};

pub const SATOSHI: u64 = 10_000_000_000;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "PascalCase", deserialize = "PascalCase"))]
pub struct Withdrawal {
    pub address: bitcoin::Address,
    pub amount: U256,
    pub fee: U256,
}

#[rpc(client)]
pub trait Ethereum {
    #[method(name = "eth_withdraw")]
    async fn withdraw(
        &self,
        from: &web3::types::Address,
        amount: &U256,
        fee: &U256,
    ) -> Result<String, jsonrpsee::core::Error>;
    #[method(name = "eth_refund")]
    async fn refund(&self, id: &H256) -> Result<H256, jsonrpsee::core::Error>;
    #[method(name = "eth_getUnspentWithdrawals")]
    async fn get_unspent_withdrawals(
        &self,
    ) -> Result<HashMap<H256, Withdrawal>, jsonrpsee::core::Error>;
}

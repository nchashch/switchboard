use jsonrpsee::proc_macros::rpc;

pub const SATOSHI: u64 = 10_000_000_000;

#[rpc(client)]
pub trait Ethereum {
    #[method(name = "eth_withdraw")]
    async fn withdraw(
        &self,
        from: &str,
        amount: u64,
        fee: u64,
    ) -> Result<String, jsonrpsee::core::Error>;
    #[method(name = "eth_refund")]
    async fn refund(&self, id: &str) -> Result<String, jsonrpsee::core::Error>;
    #[method(name = "eth_getUnspentWithdrawals")]
    async fn get_unspent_withdrawals(&self) -> Result<serde_json::Value, jsonrpsee::core::Error>;
}

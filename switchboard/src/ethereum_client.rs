use crate::amount::AmountBtc;
use jsonrpsee::proc_macros::rpc;

const SATOSHI: u64 = 10_000_000_000;

#[rpc(client)]
pub trait Ethereum {
    #[method(name = "personal_listAccounts")]
    async fn list_accounts(&self) -> Result<Vec<String>, jsonrpsee::core::Error>;
    // NOTE: Balance is in Wei not in Satoshi. There are 10^10 Wei in one Satoshi.
    #[method(name = "eth_getBalance")]
    async fn get_balance(&self, account: String) -> Result<u64, jsonrpsee::core::Error>;
    #[method(name = "eth_withdraw")]
    async fn withdraw(&self, from: String, amount: u64, fee: u64) -> Result<String, jsonrpsee::core::Error>;
}

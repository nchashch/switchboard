use crate::amount::AmountBtc;
use jsonrpsee::proc_macros::rpc;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalStatus {
    hash: bitcoin::Txid,
    nblocksleft: usize,
    nworkscore: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SpentWithdrawal {
    nsidechain: usize,
    hash: bitcoin::Txid,
    hashblock: bitcoin::BlockHash,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FailedWithdrawal {
    nsidechain: usize,
    hash: bitcoin::Txid,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vote {
    Upvote,
    Abstain,
    Downvote,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    hash: bitcoin::BlockHash,
    confirmations: usize,
    strippedsize: usize,
    size: usize,
    weight: usize,
    height: usize,
    version: i32,
    version_hex: String,
    merkleroot: bitcoin::TxMerkleNode,
    tx: Vec<bitcoin::Txid>,
    time: u32,
    mediantime: u32,
    nonce: u32,
    bits: String,
    difficulty: f64,
    chainwork: String,
    previousblockhash: Option<bitcoin::BlockHash>,
    nextblockhash: Option<bitcoin::BlockHash>,
}

#[rpc(client)]
pub trait Zcash {
    #[method(name = "stop")]
    async fn stop(&self) -> Result<String, jsonrpsee::core::Error>;
    #[method(name = "getbalance")]
    async fn getbalance(
        &self,
        account: Option<String>,
        minconf: Option<usize>,
        include_watchonly: Option<bool>,
    ) -> Result<AmountBtc, jsonrpsee::core::Error>;
    #[method(name = "getrefund")]
    async fn getrefund(
        &self,
        account: Option<String>,
        minconf: Option<usize>,
        include_watchonly: Option<bool>,
    ) -> Result<AmountBtc, jsonrpsee::core::Error>;
    #[method(name = "getnewaddress")]
    async fn getnewaddress(
        &self,
        address_type: Option<&str>,
    ) -> Result<String, jsonrpsee::core::Error>;
    #[method(name = "generate")]
    async fn generate(
        &self,
        nblocks: usize,
        amount: AmountBtc,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error>;
    #[method(name = "withdraw")]
    async fn withdraw(
        &self,
        amount: AmountBtc,
        main_fee: AmountBtc,
        main_address: Option<bitcoin::Address>,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error>;
    #[method(name = "refund")]
    async fn refund(
        &self,
        amount: AmountBtc,
        main_fee: AmountBtc,
        main_address: Option<bitcoin::Address>,
        zcash_address: Option<String>,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error>;
    #[method(name = "getblockcount")]
    async fn getblockcount(&self) -> Result<usize, jsonrpsee::core::Error>;
    #[method(name = "getblock")]
    async fn getblock(
        &self,
        blockhash: bitcoin::BlockHash,
        verbosity: Option<usize>,
    ) -> Result<Block, jsonrpsee::core::Error>;
    #[method(name = "gettransaction")]
    async fn gettransaction(
        &self,
        txid: bitcoin::Txid,
        include_watchonly: Option<bool>,
    ) -> Result<serde_json::Value, jsonrpsee::core::Error>;
    #[method(name = "getrawtransaction")]
    async fn getrawtransaction(
        &self,
        txid: bitcoin::Txid,
        verbose: Option<bool>,
        blockhash: Option<bitcoin::BlockHash>,
    ) -> Result<serde_json::Value, jsonrpsee::core::Error>;
}

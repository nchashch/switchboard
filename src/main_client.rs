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
pub trait Main {
    #[method(name = "stop")]
    async fn stop(&self) -> Result<String, jsonrpsee::core::Error>;
    #[method(name = "getbalance")]
    async fn getbalance(
        &self,
        account: Option<String>,
        minconf: Option<usize>,
        include_watchonly: Option<bool>,
    ) -> Result<AmountBtc, jsonrpsee::core::Error>;
    #[method(name = "getnewaddress")]
    async fn getnewaddress(
        &self,
        address_type: Option<&str>,
    ) -> Result<bitcoin::Address, jsonrpsee::core::Error>;
    #[method(name = "generate")]
    async fn generate(
        &self,
        nblocks: usize,
        maxtries: Option<usize>,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error>;
    #[method(name = "generatetoaddress")]
    async fn generatetoaddress(
        &self,
        nblocks: usize,
        address: bitcoin::Address,
        maxtries: Option<usize>,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error>;
    // FIXME: Define a "Deposit Address" type.
    #[method(name = "createsidechaindeposit")]
    async fn createsidechaindeposit(
        &self,
        nsidechain: usize,
        depositaddress: &str,
        amount: &AmountBtc,
        fee: &AmountBtc,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error>;
    #[method(name = "listwithdrawalstatus")]
    async fn listwithdrawalstatus(
        &self,
        nsidechain: usize,
    ) -> Result<Vec<WithdrawalStatus>, jsonrpsee::core::Error>;
    #[method(name = "listspentwithdrawals")]
    async fn listspentwithdrawals(&self) -> Result<Vec<SpentWithdrawal>, jsonrpsee::core::Error>;
    #[method(name = "listfailedwithdrawals")]
    async fn listfailedwithdrawals(&self) -> Result<Vec<FailedWithdrawal>, jsonrpsee::core::Error>;
    #[method(name = "setwithdrawalvote")]
    async fn setwithdrawalvote(
        &self,
        vote: Vote,
        nsidechain: usize,
        hashwithdrawal: bitcoin::Txid,
    ) -> Result<(), jsonrpsee::core::Error>;
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

    #[method(name = "createsidechainproposal")]
    async fn createsidechainproposal(
        &self,
        nsidechain: usize,
        title: String,
        description: Option<String>,
        version: Option<usize>,
        hashid1: Option<String>,
        hashid2: Option<String>,
    ) -> Result<serde_json::Value, jsonrpsee::core::Error>;
}

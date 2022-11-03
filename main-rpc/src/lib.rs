use amount_btc::AmountBtc;
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

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpsee::http_client::{HeaderMap, HttpClientBuilder};
    use std::str::FromStr;

    #[tokio::test]
    async fn it_works() {
        let auth = format!("{}:{}", "user", "password");
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Basic {}", base64::encode(auth)).parse().unwrap(),
        );
        let main = HttpClientBuilder::default()
            .set_headers(headers.clone())
            .build("http://localhost:18443")
            .unwrap();
        dbg!(*main.getbalance(None, None, None).await.unwrap());
        dbg!(main.getnewaddress(None).await.unwrap());
        dbg!(main.generate(1, None).await.unwrap());
        dbg!(main
            .generatetoaddress(
                1,
                bitcoin::Address::from_str("2N9BKKciG6STFC3Sho9ebePxMDUVT5uXh3c").unwrap(),
                None
            )
            .await
            .unwrap());
        dbg!(main.listwithdrawalstatus(1).await.unwrap());
        dbg!(main.listspentwithdrawals().await.unwrap());
        dbg!(main.listfailedwithdrawals().await.unwrap());
        dbg!(main
            .setwithdrawalvote(
                Vote::Abstain,
                1,
                bitcoin::Txid::from_str(
                    "5af164e1a7823e2e3ac860c59a4ecb4e7d2c114217eeea77bc9b241dc53aa2fb"
                )
                .unwrap()
            )
            .await
            .unwrap());
        dbg!(main
            .getblock(
                bitcoin::BlockHash::from_str(
                    "52b84bdffcdcc21252116f9e24fdd703a8b157cfa87e4b4ba0e2b15648a7e1c6"
                )
                .unwrap(),
                None
            )
            .await
            .unwrap());
        dbg!(main
            .gettransaction(
                bitcoin::Txid::from_str(
                    "9a71b5c02401536672b2947d7ce3200ba59cbf79427c059f549b19ae0c7632c1"
                )
                .unwrap(),
                None,
            )
            .await
            .unwrap());
        dbg!(main
            .getrawtransaction(
                bitcoin::Txid::from_str(
                    "9a71b5c02401536672b2947d7ce3200ba59cbf79427c059f549b19ae0c7632c1"
                )
                .unwrap(),
                Some(true),
                None,
            )
            .await
            .unwrap());
    }
}

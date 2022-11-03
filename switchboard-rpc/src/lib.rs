use amount_btc::AmountBtc;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use switchboard_api::{Chain, Sidechain, SidechainClient};

#[derive(Serialize, Deserialize)]
pub struct Balances {
    main: AmountBtc,
    zcash: AmountBtc,
}

impl std::fmt::Display for Balances {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "main balance:  {:>24}", format!("{}", *self.main))?;
        write!(f, "zcash balance: {:>24}", format!("{}", *self.zcash))
    }
}

impl From<switchboard_api::Balances> for Balances {
    fn from(other: switchboard_api::Balances) -> Balances {
        Balances {
            main: other.main.into(),
            zcash: other.zcash.into(),
        }
    }
}

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
        amount: AmountBtc,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error>;

    #[method(name = "getbalances")]
    async fn getbalances(&self) -> Result<Balances, jsonrpsee::core::Error>;

    #[method(name = "getnewaddress")]
    async fn getnewaddress(&self, chain: Chain) -> Result<String, jsonrpsee::core::Error>;

    #[method(name = "deposit")]
    async fn deposit(
        &self,
        sidechain: Sidechain,
        amount: AmountBtc,
        fee: AmountBtc,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error>;

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
}

#[async_trait]
impl SwitchboardRpcServer for Switchboardd {
    async fn generate(
        &self,
        number: usize,
        amount: AmountBtc,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        Ok(self.client.generate(number, *amount).await?)
    }

    async fn getbalances(&self) -> Result<Balances, jsonrpsee::core::Error> {
        Ok(self.client.get_balances().await?.into())
    }

    async fn getnewaddress(&self, chain: Chain) -> Result<String, jsonrpsee::core::Error> {
        Ok(self.client.get_new_address(chain).await?)
    }

    async fn deposit(
        &self,
        sidechain: Sidechain,
        amount: AmountBtc,
        fee: AmountBtc,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error> {
        Ok(self.client.deposit(sidechain, *amount, *fee).await?)
    }

    async fn main(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error> {
        Ok(self.client.main_request(method, params).await?)
    }

    async fn zcash(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error> {
        Ok(self.client.zcash_request(method, params).await?)
    }
}

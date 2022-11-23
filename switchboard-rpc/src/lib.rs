use amount_btc::AmountBtc;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use switchboard_api::{Chain, Sidechain, SidechainClient};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Balances {
    main: AmountBtc,
    zcash: AmountBtc,
}

impl std::fmt::Display for Balances {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", switchboard_api::Balances::from(*self))
    }
}

impl From<Balances> for switchboard_api::Balances {
    fn from(other: Balances) -> switchboard_api::Balances {
        switchboard_api::Balances {
            main: other.main.into(),
            zcash: other.zcash.into(),
        }
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

    #[method(name = "activatesidechains")]
    async fn activatesidechains(&self) -> Result<(), jsonrpsee::core::Error>;
}

#[async_trait]
impl SwitchboardRpcServer for Switchboardd {
    async fn generate(
        &self,
        number: usize,
        amount: AmountBtc,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        self.client.generate(number, *amount).await
    }

    async fn getbalances(&self) -> Result<Balances, jsonrpsee::core::Error> {
        Ok(self.client.get_balances().await?.into())
    }

    async fn getnewaddress(&self, chain: Chain) -> Result<String, jsonrpsee::core::Error> {
        self.client.get_new_address(chain).await
    }

    async fn deposit(
        &self,
        sidechain: Sidechain,
        amount: AmountBtc,
        fee: AmountBtc,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error> {
        self.client.deposit(sidechain, *amount, *fee).await
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

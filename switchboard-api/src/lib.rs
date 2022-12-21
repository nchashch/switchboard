use anyhow::Result;
use bitcoin::Amount;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use main_rpc::MainClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use switchboard_config::Config;
use zcash_rpc::ZcashClient;

#[derive(Clone)]
pub struct SidechainClient {
    main: HttpClient,
    zcash: HttpClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balances {
    pub main: u64,
    pub zcash: u64,
}

impl std::fmt::Display for Balances {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let main = Amount::from_sat(self.main);
        let zcash = Amount::from_sat(self.zcash);
        writeln!(f, "main balance:  {:>24}", format!("{}", main))?;
        write!(f, "zcash balance: {:>24}", format!("{}", zcash))
    }
}

#[derive(Copy, Clone, Debug, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Main,
    Zcash,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sidechain {
    Zcash,
}

impl std::fmt::Display for Sidechain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "zcash")
    }
}

impl Sidechain {
    fn chain(&self) -> Chain {
        match self {
            Sidechain::Zcash => Chain::Zcash,
        }
    }

    fn number(&self) -> usize {
        match self {
            Sidechain::Zcash => 0,
        }
    }
}

impl SidechainClient {
    pub fn new(config: &Config) -> Result<SidechainClient> {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", config.switchboard.basic_auth()?);
        let main = HttpClientBuilder::default()
            .set_headers(headers.clone())
            .build(config.main.socket_address())?;
        let zcash = HttpClientBuilder::default()
            .set_headers(headers)
            .build(config.zcash.socket_address())?;
        Ok(SidechainClient { main, zcash })
    }

    fn prepare_params(params: Option<Vec<Value>>) -> Option<jsonrpsee::types::ParamsSer<'static>> {
        params.map(jsonrpsee::types::ParamsSer::Array)
    }

    pub async fn main_request(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error> {
        self.main
            .request(&method, Self::prepare_params(params))
            .await
    }

    pub async fn zcash_request(
        &self,
        method: String,
        params: Option<Vec<Value>>,
    ) -> Result<Value, jsonrpsee::core::Error> {
        self.zcash
            .request(&method, Self::prepare_params(params))
            .await
    }

    pub async fn stop(&self) -> Result<Vec<String>, jsonrpsee::core::Error> {
        let zcash = ZcashClient::stop(&self.zcash).await;
        let main = MainClient::stop(&self.main).await;
        Ok(vec![zcash?, main?])
    }

    pub async fn generate(
        &self,
        number: usize,
        amount: u64,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        // FIXME: This would works for zcash and ethereum sidechains. But it
        // would be good to implement a more general solution.
        let amount = Amount::from_sat(amount);
        ZcashClient::generate(&self.zcash, number, amount.into()).await
    }

    pub async fn get_balances(&self) -> Result<Balances, jsonrpsee::core::Error> {
        let main = MainClient::getbalance(&self.main, None, None, None)
            .await?
            .to_sat();
        let zcash = ZcashClient::getbalance(&self.zcash, None, None, None)
            .await?
            .to_sat();
        Ok(Balances { main, zcash })
    }

    // FIXME: Define an enum with different kinds of addresses.
    pub async fn get_new_address(&self, chain: Chain) -> Result<String, jsonrpsee::core::Error> {
        Ok(match chain {
            Chain::Main => MainClient::getnewaddress(&self.main, None)
                .await?
                .to_string(),
            Chain::Zcash => ZcashClient::getnewaddress(&self.zcash, None)
                .await?
                .to_string(),
        })
    }

    pub async fn get_deposit_address(
        &self,
        sidechain: Sidechain,
    ) -> Result<String, jsonrpsee::core::Error> {
        let address = self.get_new_address(sidechain.chain()).await?;
        let deposit_address: String = format!("s{}_{}_", sidechain.number(), address);
        let hash = sha256::digest(deposit_address.as_bytes()).to_string();
        let hash: String = hash[..6].into();
        Ok(format!("{}{}", deposit_address, hash))
    }

    pub async fn deposit(
        &self,
        sidechain: Sidechain,
        amount: u64,
        fee: u64,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error> {
        let deposit_address = self.get_deposit_address(sidechain).await?;
        let amount = Amount::from_sat(amount);
        let fee = Amount::from_sat(fee);
        MainClient::createsidechaindeposit(
            &self.main,
            sidechain.number(),
            &deposit_address,
            &amount.into(),
            &fee.into(),
        )
        .await
    }

    pub async fn withdraw(
        &self,
        sidechain: Sidechain,
        amount: u64,
        fee: u64,
    ) -> Result<String, jsonrpsee::core::Error> {
        Ok("".into())
    }

    /// This is used for setting up a new testing environment.
    pub async fn activate_sidechains(&self) -> Result<(), jsonrpsee::core::Error> {
        let active_sidechains = [Sidechain::Zcash];
        for sidechain in active_sidechains {
            MainClient::createsidechainproposal(
                &self.main,
                sidechain.number(),
                format!("{}", sidechain),
                None,
                None,
                None,
                None,
            )
            .await?;
        }
        MainClient::generate(&self.main, 200, None).await?;
        Ok(())
    }
}

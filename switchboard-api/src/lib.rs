use amount_btc::AmountBtc;
use anyhow::Result;
pub use bitcoin::Amount;
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use mainchain_rpc::MainClient;
use switchboard_config::Config;
use zcash_rpc::ZcashClient;

#[derive(Clone)]
pub struct Client {
    main: HttpClient,
    zcash: HttpClient,
}

#[derive(Debug)]
pub struct Balances {
    pub main: Amount,
    pub zcash: Amount,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum Chain {
    Main,
    Zcash,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
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

impl Client {
    pub fn new(config: &Config) -> Result<Client> {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", config.switchboard.basic_auth()?);
        let main = HttpClientBuilder::default()
            .set_headers(headers.clone())
            .build(config.main.socket_address())?;
        let zcash = HttpClientBuilder::default()
            .set_headers(headers)
            .build(config.zcash.socket_address())?;
        Ok(Client { main, zcash })
    }

    pub async fn generate(
        &self,
        number: usize,
        amount: Amount,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        ZcashClient::generate(&self.zcash, number, AmountBtc(amount)).await
    }

    pub async fn get_balances(&self) -> Result<Balances, jsonrpsee::core::Error> {
        let main = MainClient::getbalance(&self.main, None, None, None)
            .await?
            .0;
        let zcash = ZcashClient::getbalance(&self.zcash, None, None, None)
            .await?
            .0;
        Ok(Balances { main, zcash })
    }

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
        amount: Amount,
        fee: Amount,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error> {
        let deposit_address = self.get_deposit_address(sidechain).await?;
        MainClient::createsidechaindeposit(
            &self.main,
            sidechain.number(),
            &deposit_address,
            &AmountBtc(amount),
            &AmountBtc(fee),
        )
        .await
    }
}

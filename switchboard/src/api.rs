use anyhow::Result;
use bitcoin::Amount;
use hex::ToHex;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use web3::types::{H256, U256};

use crate::config::Config;
use crate::ethereum_client::{EthereumClient, SATOSHI};
use crate::main_client::MainClient;
use crate::zcash_client::ZcashClient;

#[derive(Copy, Clone, Debug, clap::ValueEnum, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Main,
    Zcash,
    Ethereum,
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Main => write!(f, "main"),
            Chain::Zcash => write!(f, "zcash"),
            Chain::Ethereum => write!(f, "ethereum"),
        }
    }
}

#[derive(Clone)]
pub struct SidechainClient {
    main: HttpClient,
    zcash: HttpClient,
    ethereum: HttpClient,
    web3: web3::Web3<web3::transports::Http>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balances {
    main: u64,
    zcash: u64,
    ethereum: u64,

    zcash_refundable: u64,
    ethereum_refundable: u64,
}

impl std::fmt::Display for Balances {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Available balances:")?;
        writeln!(
            f,
            "main:     {:>24}",
            format!("{}", Amount::from_sat(self.main))
        );
        writeln!(
            f,
            "zcash:    {:>24}",
            format!("{}", Amount::from_sat(self.zcash))
        );
        writeln!(
            f,
            "ethereum: {:>24}",
            format!("{}", Amount::from_sat(self.ethereum))
        );
        writeln!(f, "Refundable balances:")?;
        writeln!(
            f,
            "zcash:    {:>24}",
            format!("{}", Amount::from_sat(self.zcash_refundable))
        );
        write!(
            f,
            "ethereum: {:>24}",
            format!("{}", Amount::from_sat(self.ethereum_refundable))
        )
        // FIXME: Add "pending withdrawal balances".
        //writeln!(f, "Pending withdrawal balances:");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCounts {
    pub main: usize,
    pub zcash: usize,
    pub ethereum: usize,
}

impl std::fmt::Display for BlockCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "main block count:     {:>10}", format!("{}", self.main))?;
        writeln!(f, "zcash block count:    {:>10}", format!("{}", self.zcash));
        write!(
            f,
            "ethereum block count: {:>10}",
            format!("{}", self.ethereum)
        )
    }
}

#[derive(Copy, Clone, Debug, clap::ValueEnum, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sidechain {
    Zcash,
    Ethereum,
}

impl std::fmt::Display for Sidechain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.chain().fmt(f)
    }
}

impl Sidechain {
    fn chain(&self) -> Chain {
        match self {
            Sidechain::Zcash => Chain::Zcash,
            Sidechain::Ethereum => Chain::Ethereum,
        }
    }

    fn number(&self) -> usize {
        match self {
            Sidechain::Zcash => 0,
            Sidechain::Ethereum => 1,
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
        let ethereum = HttpClientBuilder::default().build(config.ethereum.socket_address())?;
        let transport =
            web3::transports::Http::new(&format!("http://localhost:{}", config.ethereum.port))?;
        let web3 = web3::Web3::new(transport);
        Ok(SidechainClient {
            main,
            zcash,
            ethereum,
            web3,
        })
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

    pub async fn get_balances(&self) -> Result<Balances> {
        let main = MainClient::getbalance(&self.main, None, None, None)
            .await?
            .to_sat();
        let zcash = ZcashClient::getbalance(&self.zcash, None, None, None)
            .await?
            .to_sat();
        let ethereum = {
            let accounts = self.web3.eth().accounts().await?;
            let mut balance = U256::zero();
            for account in accounts.iter() {
                balance += self.web3.eth().balance(*account, None).await?;
            }
            (balance / SATOSHI).as_u64()
        };
        let zcash_refundable = ZcashClient::getrefund(&self.zcash, None, None, None)
            .await?
            .to_sat();
        let ethereum_refundable = {
            let unspent_withdrawals =
                EthereumClient::get_unspent_withdrawals(&self.ethereum).await?;
            unspent_withdrawals
                .values()
                .map(|uw| (uw.amount / SATOSHI).as_u64())
                .sum()
        };
        Ok(Balances {
            main,
            zcash,
            ethereum,

            zcash_refundable,
            ethereum_refundable,
        })
    }

    pub async fn get_block_counts(&self) -> Result<BlockCounts> {
        let main = MainClient::getblockcount(&self.main).await?;
        let zcash = ZcashClient::getblockcount(&self.zcash).await?;
        let ethereum = self.web3.eth().block_number().await?.as_usize();
        Ok(BlockCounts {
            main,
            zcash,
            ethereum,
        })
    }

    async fn get_ethereum_account(&self) -> Result<web3::types::Address> {
        self.web3
            .eth()
            .accounts()
            .await?
            .first()
            .ok_or(anyhow::Error::msg("No available Ethereum addresses"))
            .copied()
    }

    // FIXME: Define an enum with different kinds of addresses.
    pub async fn get_new_address(&self, chain: Chain) -> Result<String> {
        Ok(match chain {
            Chain::Main => MainClient::getnewaddress(&self.main, None)
                .await?
                .to_string(),
            Chain::Zcash => ZcashClient::getnewaddress(&self.zcash, None)
                .await?
                .to_string(),
            Chain::Ethereum => format!(
                "0x{}",
                self.get_ethereum_account().await?.encode_hex::<String>()
            ),
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

    pub async fn withdraw(&self, sidechain: Sidechain, amount: u64, fee: u64) -> Result<String> {
        let amount = Amount::from_sat(amount);
        let fee = Amount::from_sat(fee);
        let id = match sidechain {
            Sidechain::Zcash => ZcashClient::withdraw(&self.zcash, amount.into(), fee.into(), None)
                .await?
                .to_string(),
            Sidechain::Ethereum => {
                let account = self.get_ethereum_account().await?;
                let amount: U256 = (amount.to_sat()).into();
                let fee: U256 = (fee.to_sat()).into();
                EthereumClient::withdraw(&self.ethereum, &account, &amount, &fee).await?
            }
        };
        Ok(id)
    }

    pub async fn refund(&self, sidechain: Sidechain, amount: u64, fee: u64) -> Result<()> {
        match sidechain {
            Sidechain::Zcash => {
                let amount = Amount::from_sat(amount);
                let fee = Amount::from_sat(fee);
                ZcashClient::refund(&self.zcash, amount.into(), fee.into(), None, None).await?;
            }
            Sidechain::Ethereum => {
                let mut unspent_withdrawals: Vec<(H256, U256)> =
                    EthereumClient::get_unspent_withdrawals(&self.ethereum)
                        .await?
                        .iter()
                        .map(|(id, uw)| (*id, uw.amount))
                        .collect();
                unspent_withdrawals.sort_by(|a, b| a.cmp(b));
                let mut wei_amount: U256 = amount.into();
                wei_amount *= SATOSHI;
                let mut total_refund = U256::zero();
                let mut refunded_withdrawals = HashSet::new();
                dbg!(&unspent_withdrawals);
                for (id, refund) in unspent_withdrawals.iter() {
                    if total_refund > wei_amount {
                        break;
                    }
                    total_refund += *refund;
                    refunded_withdrawals.insert(id);
                }
                if total_refund < wei_amount {
                    return Err(anyhow::Error::msg(
                        "not enough funds in unspent withdrawals to refund",
                    ));
                }
                let wei_change = total_refund - wei_amount;
                for id in refunded_withdrawals.iter() {
                    dbg!(id);
                    EthereumClient::refund(&self.ethereum, id).await?;
                }
                let account = self.get_ethereum_account().await?;
                let change: U256 = wei_change / SATOSHI;
                let fee: U256 = fee.into();
                // FIXME: Handle dust here.
                if change > U256::zero() {
                    EthereumClient::withdraw(&self.ethereum, &account, &change, &fee).await?;
                }
            }
        };
        Ok(())
    }

    /// This is used for setting up a new testing environment.
    pub async fn activate_sidechains(&self) -> Result<(), jsonrpsee::core::Error> {
        let active_sidechains = [Sidechain::Zcash, Sidechain::Ethereum];
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

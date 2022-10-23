pub use amount_btc::AmountBtc;
use anyhow::Result;
use jsonrpsee::http_client::{HeaderMap, HttpClient, HttpClientBuilder};
use mainchain_rpc::MainClient;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zcash_rpc::ZcashClient;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MainConfig {
    bin: PathBuf,
    port: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ZcashConfig {
    bin: PathBuf,
    port: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SwitchboardConfig {
    datadir: PathBuf,
    // Is it ok to reuse the same rpcuser and rpcpassword for all sidechains?
    rpcuser: String,
    rpcpassword: String,
    regtest: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    switchboard: SwitchboardConfig,
    main: MainConfig,
    zcash: ZcashConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            switchboard: SwitchboardConfig {
                datadir: "../data".into(),
                rpcuser: "user".into(),
                rpcpassword: "password".into(),
                regtest: true,
            },
            main: MainConfig {
                bin: "../mainchain/src/drivechaind".into(),
                port: 18443,
            },
            zcash: ZcashConfig {
                bin: "../zcash-sidechain/src/zcashd".into(),
                port: 19443,
            },
        }
    }
}

#[derive(Clone)]
pub struct Node {
    config: Config,
    main: HttpClient,
    zcash: HttpClient,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Balances {
    pub main: AmountBtc,
    pub zcash: AmountBtc,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Chain {
    Main,
    Zcash,
}

#[derive(Clone, Debug, clap::ValueEnum)]
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

impl Node {
    pub fn new(config: Config) -> Result<Node> {
        let mut headers = HeaderMap::new();
        let auth = format!(
            "{}:{}",
            config.switchboard.rpcuser, config.switchboard.rpcpassword
        );
        headers.insert(
            "authorization",
            format!("Basic {}", base64::encode(auth)).parse().unwrap(),
        );
        let main = HttpClientBuilder::default()
            .set_headers(headers.clone())
            .build(format!("http://localhost:{}", config.main.port))?;
        let mut headers = HeaderMap::new();
        let auth = format!(
            "{}:{}",
            config.switchboard.rpcuser, config.switchboard.rpcpassword
        );
        headers.insert(
            "authorization",
            format!("Basic {}", base64::encode(auth)).parse().unwrap(),
        );
        let zcash = HttpClientBuilder::default()
            .set_headers(headers.clone())
            .build(format!("http://localhost:{}", config.zcash.port))?;
        Ok(Node {
            config,
            main,
            zcash,
        })
    }

    pub async fn generate(
        &self,
        number: usize,
        amount: bitcoin::Amount,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        ZcashClient::generate(&self.zcash, number, AmountBtc(amount)).await
    }

    pub async fn get_balances(&self) -> Result<Balances, jsonrpsee::core::Error> {
        let main = MainClient::getbalance(&self.main, None, None, None).await?;
        let zcash = ZcashClient::getbalance(&self.zcash, None, None, None).await?;
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
        sidechain: &Sidechain,
    ) -> Result<String, jsonrpsee::core::Error> {
        let address = self.get_new_address(sidechain.chain()).await?;
        let deposit_address: String = format!("s{}_{}_", sidechain.number(), address);
        let hash = sha256::digest(deposit_address.as_bytes()).to_string();
        let hash: String = hash[..6].into();
        Ok(format!("{}{}", deposit_address, hash))
    }

    pub async fn deposit(
        &self,
        sidechain: &Sidechain,
        amount: AmountBtc,
        fee: AmountBtc,
    ) -> Result<bitcoin::Txid, jsonrpsee::core::Error> {
        let deposit_address = self.get_deposit_address(&sidechain).await?;
        MainClient::createsidechaindeposit(
            &self.main,
            sidechain.number(),
            &deposit_address,
            &amount,
            &fee,
        )
        .await
    }

    pub async fn run_daemons(&self) -> Result<()> {
        let mut mainchain = std::process::Command::new(&self.config.main.bin);
        let mut zcash = std::process::Command::new(&self.config.zcash.bin);
        let mut main_process = mainchain
            .arg(format!(
                "-datadir={}",
                mainchain_datadir(&self.config.switchboard.datadir).display()
            ))
            .arg(format!("-rpcport={}", self.config.main.port))
            .arg(format!("-rpcuser={}", self.config.switchboard.rpcuser))
            .arg(format!(
                "-rpcpassword={}",
                self.config.switchboard.rpcpassword
            ))
            .arg(format!(
                "-regtest={}",
                match self.config.switchboard.regtest {
                    true => 1,
                    false => 0,
                }
            ))
            .spawn()?;
        let mut zcash_process = zcash
            .arg(format!(
                "-datadir={}",
                zcash_datadir(&self.config.switchboard.datadir).display()
            ))
            .arg(format!("-rpcport={}", self.config.zcash.port))
            .arg(format!("-rpcuser={}", self.config.switchboard.rpcuser))
            .arg(format!(
                "-rpcpassword={}",
                self.config.switchboard.rpcpassword
            ))
            .arg(format!(
                "-regtest={}",
                match self.config.switchboard.regtest {
                    true => 1,
                    false => 0,
                }
            ))
            .spawn()?;

        let (tx, rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || {
            tx.send(()).unwrap();
        })?;
        rx.recv()?;
        signal::kill(Pid::from_raw(zcash_process.id() as i32), Signal::SIGINT).unwrap();
        signal::kill(Pid::from_raw(main_process.id() as i32), Signal::SIGINT).unwrap();
        zcash_process.wait()?;
        main_process.wait()?;
        Ok(())
    }
}

fn mainchain_datadir(datadir: &Path) -> PathBuf {
    datadir.join(Path::new("main"))
}

fn zcash_datadir(datadir: &Path) -> PathBuf {
    datadir.join(Path::new("zcash"))
}

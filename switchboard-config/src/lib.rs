use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChainConfig {
    pub bin: PathBuf,
    pub host: String,
    pub port: u16,
}

impl ChainConfig {
    pub fn socket_address(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SwitchboardConfig {
    pub datadir: PathBuf,
    // Is it ok to reuse the same rpcuser and rpcpassword for all sidechains?
    pub rpcuser: String,
    pub rpcpassword: String,
    pub regtest: bool,
}

impl SwitchboardConfig {
    pub fn basic_auth(&self) -> Result<http::HeaderValue, http::header::InvalidHeaderValue> {
        let auth = format!("{}:{}", self.rpcuser, self.rpcpassword);
        let header_value = format!("Basic {}", base64::encode(auth)).parse()?;
        Ok(header_value)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub switchboard: SwitchboardConfig,
    pub main: ChainConfig,
    pub zcash: ChainConfig,
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
            main: ChainConfig {
                bin: "../mainchain/src/drivechaind".into(),
                host: "localhost".into(),
                port: 18443,
            },
            zcash: ChainConfig {
                bin: "../zcash-sidechain/src/zcashd".into(),
                host: "localhost".into(),
                port: 19443,
            },
        }
    }
}

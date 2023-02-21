use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChainConfig {
    pub bin: Option<PathBuf>,
    pub port: u16,
}

impl ChainConfig {
    pub fn socket_address(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SwitchboardConfig {
    // Is it ok to reuse the same rpcuser and rpcpassword for all sidechains?
    pub rpcuser: String,
    pub rpcpassword: String,
    pub regtest: bool,

    pub port: u16,
}

impl SwitchboardConfig {
    pub fn basic_auth(&self) -> Result<http::HeaderValue, http::header::InvalidHeaderValue> {
        let auth = format!("{}:{}", self.rpcuser, self.rpcpassword);
        let header_value = format!("Basic {}", base64::encode(auth)).parse()?;
        Ok(header_value)
    }

    pub fn socket_address(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("127.0.0.1:{}", self.port).parse()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub switchboard: SwitchboardConfig,
    pub main: ChainConfig,
    pub zcash: ChainConfig,
    pub ethereum: ChainConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            switchboard: SwitchboardConfig {
                rpcuser: "user".into(),
                rpcpassword: "password".into(),
                regtest: true,
                port: 20443,
            },
            main: ChainConfig {
                port: 18443,
                bin: None,
            },
            zcash: ChainConfig {
                port: 19000,
                bin: None,
            },
            ethereum: ChainConfig {
                port: 19001,
                bin: None,
            },
        }
    }
}

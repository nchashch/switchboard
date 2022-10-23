use anyhow::Result;
use std::path::{Path, PathBuf};
use switchboard_config::Config;

pub struct Daemons {
    pub main: std::process::Child,
    pub zcash: std::process::Child,
}

pub async fn spawn_daemons(config: &Config) -> Result<Daemons> {
    let main = std::process::Command::new(&config.main.bin)
        .arg(format!(
            "-datadir={}",
            mainchain_datadir(&config.switchboard.datadir).display()
        ))
        .arg(format!("-rpcport={}", config.main.port))
        .arg(format!("-rpcuser={}", config.switchboard.rpcuser))
        .arg(format!("-rpcpassword={}", config.switchboard.rpcpassword))
        .arg(format!(
            "-regtest={}",
            match config.switchboard.regtest {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    let zcash = std::process::Command::new(&config.zcash.bin)
        .arg(format!(
            "-datadir={}",
            zcash_datadir(&config.switchboard.datadir).display()
        ))
        .arg(format!("-rpcport={}", config.zcash.port))
        .arg(format!("-rpcuser={}", config.switchboard.rpcuser))
        .arg(format!("-rpcpassword={}", config.switchboard.rpcpassword))
        .arg(format!(
            "-regtest={}",
            match config.switchboard.regtest {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(Daemons { main, zcash })
}

fn mainchain_datadir(datadir: &Path) -> PathBuf {
    datadir.join(Path::new("main"))
}

fn zcash_datadir(datadir: &Path) -> PathBuf {
    datadir.join(Path::new("zcash"))
}

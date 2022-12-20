use anyhow::Result;
use std::path::Path;
use switchboard_api::SidechainClient;
use switchboard_config::Config;

pub struct Daemons {
    pub main: tokio::process::Child,
    pub zcash: tokio::process::Child,
}

async fn spawn_main(datadir: &Path, config: &Config) -> Result<tokio::process::Child> {
    let main_datadir = datadir.join("data/main");
    std::fs::create_dir_all(&main_datadir)?;
    let main = tokio::process::Command::new(datadir.join("bin/drivechaind"))
        .arg(format!("-datadir={}", main_datadir.display()))
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
    Ok(main)
}

async fn spawn_zcash(datadir: &Path, config: &Config) -> Result<tokio::process::Child> {
    let zcash_datadir = datadir.join("data/zcash");
    std::fs::create_dir_all(&zcash_datadir)?;
    let zcash_conf_path = zcash_datadir.join("zcash.conf");
    let zcash_conf = "nuparams=5ba81b19:1
nuparams=76b809bb:1";
    std::fs::write(zcash_conf_path, zcash_conf)?;
    let zcash = tokio::process::Command::new(datadir.join("bin/zcashd"))
        .arg(format!("-datadir={}", zcash_datadir.display()))
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
    Ok(zcash)
}

pub async fn spawn_daemons(datadir: &Path, config: &Config) -> Result<Daemons> {
    std::fs::create_dir_all(&datadir)?;
    let main = spawn_main(datadir, config).await;
    let zcash = spawn_zcash(datadir, config).await;
    if [&main, &zcash].iter().any(|r| r.is_err()) {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let client = SidechainClient::new(config)?;
        client.stop().await?;
        main.as_ref().unwrap();
        zcash.as_ref().unwrap();
    }
    let main = main?;
    let zcash = zcash?;
    Ok(Daemons { main, zcash })
}

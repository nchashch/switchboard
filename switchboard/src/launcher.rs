use crate::api::SidechainClient;
use crate::config::Config;
use anyhow::Result;
use bytes::Buf;
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::Path;
use tar::Archive;

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
    }
    let main = main?;
    let zcash = zcash?;
    Ok(Daemons { main, zcash })
}

pub async fn download_binaries(datadir: &Path, url: &str) -> Result<()> {
    download(
        url,
        datadir,
        "c0ec39fea69cafee61208970129c780ab300c67766881597d332db86f3be4aec",
    )
    .await?;
    Ok(())
}

pub async fn download(url: &str, path: &Path, digest: &str) -> Result<()> {
    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;
    assert_eq!(sha256::digest(content.as_ref()), digest);
    let tar = GzDecoder::new(content.reader());
    let mut archive = Archive::new(tar);
    archive.unpack(path)?;
    Ok(())
}

// drivechain linux 5d6d1a6f338038cd620606ff898b337f7973c2a61cf364118770a1acf47c2b94

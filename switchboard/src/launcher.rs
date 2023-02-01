use crate::api::SidechainClient;
use crate::config::Config;
use anyhow::Result;
use bytes::Buf;
use flate2::read::GzDecoder;
use std::path::Path;
use tar::Archive;

pub struct Daemons {
    pub main: tokio::process::Child,
    pub zcash: tokio::process::Child,
    pub ethereum: tokio::process::Child,
}

fn spawn_main(datadir: &Path, config: &Config) -> Result<tokio::process::Child> {
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

fn spawn_zcash(datadir: &Path, config: &Config) -> Result<tokio::process::Child> {
    let zcash_datadir = datadir.join("data/zcash");
    std::fs::create_dir_all(&zcash_datadir)?;
    let zcash_conf_path = zcash_datadir.join("zcash.conf");
    let zcash_conf = "nuparams=5ba81b19:1
nuparams=76b809bb:1";
    std::fs::write(zcash_conf_path, zcash_conf)?;
    let zcash = tokio::process::Command::new(datadir.join("bin/zcashd"))
        .arg(format!("-datadir={}", zcash_datadir.display()))
        .arg(format!("-mainport={}", config.main.port))
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

fn spawn_ethereum(datadir: &Path, config: &Config) -> Result<tokio::process::Child> {
    let ethereum_datadir = datadir.join("data/ethereum");
    std::fs::create_dir_all(&ethereum_datadir)?;
    let ethereum_conf_path = ethereum_datadir.join("ethereum.conf");
    let ethereum = tokio::process::Command::new(datadir.join("bin/geth"))
        .arg(format!("--datadir={}", ethereum_datadir.display()))
        .arg(format!("--http.port={}", config.ethereum.port))
        .arg(format!("--main.port={}", config.main.port))
        .arg("--http")
        .arg("--http.api=eth,web3,personal,net")
        .arg("--maxpeers=0")
        .arg("--dev")
        .spawn()?;
    Ok(ethereum)
}

pub async fn spawn_daemons(datadir: &Path, config: &Config) -> Result<Daemons> {
    std::fs::create_dir_all(datadir)?;
    let main = spawn_main(datadir, config);
    let zcash = spawn_zcash(datadir, config);
    // FIXME: This is a temporary hack to ensure geth launches after mainchain.
    // If mainchain isn't running when geth is launched, geth crashes.
    std::thread::sleep(std::time::Duration::from_secs(3));
    let ethereum = spawn_ethereum(datadir, config);
    if [&main, &zcash, &ethereum].iter().any(|r| r.is_err()) {
        let client = SidechainClient::new(config)?;
        client.stop().await?;
        let ethereum_pid = ethereum.as_ref().unwrap().id().unwrap();
        tokio::process::Command::new("kill")
            .args(["-s", "HUP", &ethereum_pid.to_string()])
            .spawn()?
            .wait()
            .await?;
    }
    let main = main?;
    let zcash = zcash?;
    let ethereum = ethereum?;
    Ok(Daemons {
        main,
        zcash,
        ethereum,
    })
}

pub async fn download_binaries(datadir: &Path, url: &str) -> Result<()> {
    download(
        url,
        datadir,
        "49d3f4bdfbc98c0b93a18869bed917497620a917f3b3e75b455c4d16582f8ec3",
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

pub async fn ethereum_regtest_setup(datadir: &Path) -> Result<()> {
    const genesis_json: &str = r#"
{
"config": {
    "chainId": 1337,
    "homesteadBlock": 0,
    "eip150Block": 0,
    "eip155Block": 0,
    "eip158Block": 0,
    "byzantiumBlock": 0,
    "constantinopleBlock": 0,
    "petersburgBlock": 0,
    "istanbulBlock": 0,
    "berlinBlock": 0
},
"difficulty": "0",
"gasLimit": "21000000",
"alloc": {
    "0xc96aaa54e2d44c299564da76e1cd3184a2386b8d":
    { "balance": "21000000000000000000000000"}
}
}
"#;

    let ethereum_datadir = datadir.join("data/ethereum");
    std::fs::create_dir_all(&ethereum_datadir)?;
    let genesis_json_path = ethereum_datadir.join("genesis.json");
    std::fs::write(&genesis_json_path, genesis_json);
    let ethereum = tokio::process::Command::new(datadir.join("bin/geth"))
        .arg(format!("--datadir={}", ethereum_datadir.display()))
        .arg("init")
        .arg(format!("{}", genesis_json_path.display()))
        .spawn()?;
    Ok(())
}

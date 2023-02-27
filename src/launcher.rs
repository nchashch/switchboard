use crate::config::Config;
use anyhow::Result;
use bytes::Buf;
use flate2::read::GzDecoder;
use std::path::Path;
use tar::Archive;
use ureq_jsonrpc::json;

pub fn spawn_bitassets_qt(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let main_datadir = datadir.join("data/main");
    let bitassets_datadir = datadir.join("data/bitassets");
    std::fs::create_dir_all(&bitassets_datadir)?;
    let default_bin = &datadir.join("bin/bitassets-qt");
    let bin = config.bitassets.bin.as_ref().unwrap_or(default_bin);
    let bitassets = std::process::Command::new(bin)
        .arg("-server=1")
        .arg(format!("-drivechain-datadir={}", main_datadir.display()))
        .arg(format!("-datadir={}", bitassets_datadir.display()))
        .arg(format!("-rpcport={}", config.bitassets.port))
        .arg(format!("-rpcuser={}", config.switchboard.rpcuser))
        .arg(format!("-rpcpassword={}", config.switchboard.rpcpassword))
        .arg(format!(
            "-regtest={}",
            match config.switchboard.regtest {
                true => 1,
                false => 0,
            }
        ))
        .arg(format!(
            "-printtoconsole={}",
            match config.zcash.verbose {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(bitassets)
}

pub fn spawn_testchain_qt(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let main_datadir = datadir.join("data/main");
    let testchain_datadir = datadir.join("data/testchain");
    std::fs::create_dir_all(&testchain_datadir)?;
    let default_bin = &datadir.join("bin/testchain-qt");
    let bin = config.testchain.bin.as_ref().unwrap_or(default_bin);
    let testchain = std::process::Command::new(bin)
        .arg("-server=1")
        .arg(format!("-drivechain-datadir={}", main_datadir.display()))
        .arg(format!("-datadir={}", testchain_datadir.display()))
        .arg(format!("-rpcport={}", config.testchain.port))
        .arg(format!("-rpcuser={}", config.switchboard.rpcuser))
        .arg(format!("-rpcpassword={}", config.switchboard.rpcpassword))
        .arg(format!(
            "-regtest={}",
            match config.switchboard.regtest {
                true => 1,
                false => 0,
            }
        ))
        .arg(format!(
            "-printtoconsole={}",
            match config.zcash.verbose {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(testchain)
}

pub fn spawn_main_qt(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let main_datadir = datadir.join("data/main");
    std::fs::create_dir_all(&main_datadir)?;
    let default_bin = &datadir.join("bin/drivechain-qt");
    let bin = config.main.bin.as_ref().unwrap_or(default_bin);
    let main = std::process::Command::new(bin)
        .arg("-server=1")
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
        .arg(format!(
            "-printtoconsole={}",
            match config.zcash.verbose {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(main)
}

pub fn spawn_main(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let main_datadir = datadir.join("data/main");
    std::fs::create_dir_all(&main_datadir)?;
    let default_bin = &datadir.join("bin/drivechaind");
    let bin = config.main.bin.as_ref().unwrap_or(default_bin);
    let main = std::process::Command::new(bin)
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
        .arg(format!(
            "-printtoconsole={}",
            match config.zcash.verbose {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(main)
}

pub fn spawn_zcash(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let zcash_datadir = datadir.join("data/zcash");
    std::fs::create_dir_all(&zcash_datadir)?;
    let zcash_conf_path = zcash_datadir.join("zcash.conf");
    let zcash_conf = "nuparams=5ba81b19:1
nuparams=76b809bb:1";
    std::fs::write(zcash_conf_path, zcash_conf)?;
    let default_bin = &datadir.join("bin/zcashd");
    let bin = config.zcash.bin.as_ref().unwrap_or(default_bin);
    let zcash = std::process::Command::new(bin)
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
        .arg(format!(
            "-printtoconsole={}",
            match config.zcash.verbose {
                true => 1,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(zcash)
}

pub fn spawn_ethereum(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let ethereum_datadir = datadir.join("data/ethereum");
    std::fs::create_dir_all(&ethereum_datadir)?;
    let default_bin = &datadir.join("bin/geth");
    let bin = config.ethereum.bin.as_ref().unwrap_or(default_bin);
    let ethereum = std::process::Command::new(bin)
        .arg(format!("--datadir={}", ethereum_datadir.display()))
        .arg(format!("--http.port={}", config.ethereum.port))
        .arg(format!("--main.port={}", config.main.port))
        .arg("--http")
        .arg("--http.api=eth,web3,personal,net")
        .arg("--maxpeers=0")
        .arg("--dev")
        .arg(format!(
            "--verbosity={}",
            match config.ethereum.verbose {
                true => 3,
                false => 0,
            }
        ))
        .spawn()?;
    Ok(ethereum)
}

pub fn download_binaries(datadir: &Path, url: &str) -> Result<()> {
    const SHA256_DIGEST: &str = "adeca73e0b5e08e74b4ef20c057319bcc85fab8453deee677b74c060d3e89e29";
    download(url, datadir, SHA256_DIGEST)?;
    Ok(())
}

pub fn download(url: &str, path: &Path, digest: &str) -> Result<()> {
    let resp = ureq::get(url).call()?;
    let len: usize = resp.header("Content-Length").unwrap().parse()?;
    let mut content: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader().read_to_end(&mut content)?;
    assert_eq!(sha256::digest(content.as_slice()), digest);
    let tar = GzDecoder::new(content.reader());
    let mut archive = Archive::new(tar);
    archive.unpack(path)?;
    Ok(())
}

pub fn zcash_fetch_params(datadir: &Path) -> Result<()> {
    std::process::Command::new(datadir.join("bin/fetch-params.sh"))
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn ethereum_regtest_setup(datadir: &Path) -> Result<()> {
    const GENESIS_JSON: &str = r#"
{
"config": {
    "chainId": 5123,
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
    std::fs::write(&genesis_json_path, GENESIS_JSON)?;
    std::process::Command::new(datadir.join("bin/geth"))
        .arg(format!("--datadir={}", ethereum_datadir.display()))
        .arg("init")
        .arg(format!("{}", genesis_json_path.display()))
        .spawn()?
        .wait()?;
    Ok(())
}

/// This is used for setting up a new testing environment.
pub fn activate_sidechains(main_client: &ureq_jsonrpc::Client) -> Result<(), ureq_jsonrpc::Error> {
    let active_sidechains = [
        (0, "testchain"),
        (4, "bitassets"),
        (5, "zcash"),
        (6, "ethereum"),
    ];
    for (sidechain_number, sidechain_name) in active_sidechains {
        main_client.send_request::<ureq_jsonrpc::Value>(
            "createsidechainproposal",
            &[json!(sidechain_number), json!(sidechain_name)],
        )?;
    }
    main_client.send_request::<ureq_jsonrpc::Value>("generate", &[json!(200)])?;
    Ok(())
}

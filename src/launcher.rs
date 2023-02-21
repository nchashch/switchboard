use crate::client::SidechainClient;
use crate::config::Config;
use anyhow::Result;
use bytes::Buf;
use flate2::read::GzDecoder;
use std::path::Path;
use tar::Archive;

pub struct Daemons {
    client: SidechainClient,
    main: std::process::Child,
    zcash: std::process::Child,
    ethereum: std::process::Child,
}

impl Daemons {
    pub fn start(url: &str, datadir: &Path, config: &Config) -> Result<Daemons> {
        let mut first_launch = false;
        if !datadir.join("bin").exists() {
            download_binaries(&datadir, url)?;
            if config.switchboard.regtest {
                ethereum_regtest_setup(&datadir)?;
            }
            first_launch = true;
        }
        let home_dir = dirs::home_dir().unwrap();
        if !home_dir.join(".zcash-params").exists() {
            zcash_fetch_params(&datadir)?;
        }
        let daemons = Self::spawn(datadir, config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        if config.switchboard.regtest && first_launch {
            daemons.client.activate_sidechains()?;
        }
        Ok(daemons)
    }

    pub fn stop(&mut self) -> Result<()> {
        self.client.stop()?;
        self.zcash.wait()?;
        self.main.wait()?;
        self.ethereum.wait()?;
        Ok(())
    }

    fn spawn(datadir: &Path, config: &Config) -> Result<Daemons> {
        std::fs::create_dir_all(datadir)?;
        let main = spawn_main(datadir, config);
        let zcash = spawn_zcash(datadir, config);
        // FIXME: This is a temporary hack to ensure geth launches after mainchain.
        // If mainchain isn't running when geth is launched, geth crashes.
        std::thread::sleep(std::time::Duration::from_secs(3));
        let ethereum = spawn_ethereum(datadir, config);
        let client = SidechainClient::new(config)?;
        if [&main, &zcash, &ethereum].iter().any(|r| r.is_err()) {
            client.stop()?;
            let ethereum_pid = ethereum.as_ref().unwrap().id();
            std::process::Command::new("kill")
                .args(["-s", "HUP", &ethereum_pid.to_string()])
                .spawn()?
                .wait()?;
        }
        let main = main?;
        let zcash = zcash?;
        let ethereum = ethereum?;
        Ok(Daemons {
            client,
            main,
            zcash,
            ethereum,
        })
    }
}

fn spawn_main(datadir: &Path, config: &Config) -> Result<std::process::Child> {
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
        .spawn()?;
    Ok(main)
}

fn spawn_zcash(datadir: &Path, config: &Config) -> Result<std::process::Child> {
    let zcash_datadir = datadir.join("data/zcash");
    std::fs::create_dir_all(&zcash_datadir)?;
    let zcash_conf_path = zcash_datadir.join("zcash.conf");
    let zcash_conf = "nuparams=5ba81b19:1
nuparams=76b809bb:1
printtoconsole=1";
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
        .spawn()?;
    Ok(zcash)
}

fn spawn_ethereum(datadir: &Path, config: &Config) -> Result<std::process::Child> {
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
        .arg("--verbosity=0")
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
        .spawn()?;
    Ok(())
}

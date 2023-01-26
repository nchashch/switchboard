use anyhow::Result;
use clap::Parser;
use jsonrpsee::http_server::HttpServerBuilder;
use std::net::SocketAddr;
use std::path::PathBuf;
use switchboard::{
    api::SidechainClient,
    config::Config,
    launcher::*,
    server::{SwitchboardRpcServer, Switchboardd},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    datadir: Option<PathBuf>,
    #[arg(short, long)]
    bin_download_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let home_dir = dirs::home_dir().unwrap();
    let datadir = args
        .datadir
        .unwrap_or_else(|| home_dir.join(".switchboard"));
    let config: Config = confy::load_path(datadir.join("config.toml"))?;
    if !datadir.join("bin").exists() {
        let url = args
            .bin_download_url
            .unwrap_or("http://localhost:8080/bin.tar.gz".into());
        switchboard::launcher::download_binaries(&datadir, &url).await?;
    }
    let client = SidechainClient::new(&config)?;
    let Daemons {
        mut main,
        mut zcash,
    } = spawn_daemons(&datadir, &config).await?;
    run_server(&config, &client).await?;
    client.stop().await?;
    zcash.wait().await?;
    main.wait().await?;
    Ok(())
}

async fn run_server(config: &Config, client: &SidechainClient) -> anyhow::Result<SocketAddr> {
    let server = HttpServerBuilder::default()
        .build(config.switchboard.socket_address()?)
        .await?;
    let addr = server.local_addr()?;
    server
        .start(Switchboardd::new(client.clone()).into_rpc())?
        .await;
    Ok(addr)
}

use anyhow::Result;
use clap::Parser;
use jsonrpsee::http_server::HttpServerBuilder;
use std::net::SocketAddr;
use std::path::PathBuf;
use switchboard_config::Config;
use switchboard_launcher::*;
use switchboard_rpc::{SwitchboardRpcServer, Switchboardd};
use switchboard_api::SidechainClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config_path = match args.config_path {
        Some(config_path) => config_path,
        None => "./config.toml".into(),
    };
    let config: Config = confy::load_path(config_path)?;
    let client = SidechainClient::new(&config)?;
    let Daemons {
        mut main,
        mut zcash,
    } = spawn_daemons(&config).await?;
    run_server(&config).await?;
    client.stop().await?;
    zcash.wait().await?;
    main.wait().await?;
    Ok(())
}

async fn run_server(config: &Config) -> anyhow::Result<SocketAddr> {
    let server = HttpServerBuilder::default()
        .build(config.switchboard.socket_address()?)
        .await?;
    let addr = server.local_addr()?;
    server.start(Switchboardd.into_rpc())?.await;
    Ok(addr)
}

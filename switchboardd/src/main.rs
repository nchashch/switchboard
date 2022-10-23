use anyhow::Result;
use clap::Parser;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::path::PathBuf;
use switchboard_config::Config;
use switchboard_launcher::*;

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
    let Daemons {
        mut main,
        mut zcash,
    } = spawn_daemons(&config).await?;

    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).unwrap();
    })?;
    rx.recv()?;
    signal::kill(Pid::from_raw(zcash.id() as i32), Signal::SIGINT).unwrap();
    signal::kill(Pid::from_raw(main.id() as i32), Signal::SIGINT).unwrap();
    zcash.wait()?;
    main.wait()?;
    Ok(())
}

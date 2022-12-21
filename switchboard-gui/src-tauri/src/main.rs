#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use anyhow::Result;
use clap::Parser;
use futures::executor::block_on;
use std::path::PathBuf;
use switchboard_api::{Balances, SidechainClient};
use switchboard_config::Config;
use switchboard_launcher::*;
use tauri::{RunEvent, WindowEvent};

#[tauri::command]
async fn generate(
    client: tauri::State<'_, SidechainClient>,
    amount: u64,
) -> Result<String, String> {
    client
        .generate(1, amount)
        .await
        .map(|hashes| hashes.first().unwrap().to_string())
        .map_err(|err| format!("{:#?}", err))
}

#[tauri::command]
async fn get_balances(client: tauri::State<'_, SidechainClient>) -> Result<Balances, String> {
    let balances = client
        .get_balances()
        .await
        .map_err(|err| format!("{:#?}", err))?;
    Ok(balances)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let home_dir = dirs::home_dir().unwrap();
    // let sb_dir = home_dir.join(".switchboard");
    let sb_dir = home_dir.join("dev/data/switchboard");
    let datadir = args.datadir.unwrap_or(sb_dir);
    let config: Config = confy::load_path(datadir.join("config.toml"))?;
    let client = SidechainClient::new(&config)?;
    spawn_daemons(&datadir, &config).await?;
    let app = tauri::Builder::default()
        .manage(client.clone())
        .invoke_handler(tauri::generate_handler![generate, get_balances])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(move |_app_handle, event| match event {
        tauri::RunEvent::Exit => {
            block_on(client.stop()).unwrap();
        }
        _ => {}
    });
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    datadir: Option<PathBuf>,
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use switchboard_api::*;
use switchboard_config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config_path: Option<PathBuf>,
    #[command(subcommand)]
    commands: Commands,
}

fn btc_amount_parser(s: &str) -> Result<bitcoin::Amount, bitcoin::util::amount::ParseAmountError> {
    bitcoin::Amount::from_str_in(s, bitcoin::Denomination::Bitcoin)
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a mainchain block
    Generate {
        number: usize,
        #[arg(value_parser = btc_amount_parser)]
        amount: bitcoin::Amount,
    },
    /// Get balances for mainchain and all sidechains
    Getbalances,
    /// Get a new address
    Getnewaddress { chain: Chain },
    /// Create a deposit to a sidechain
    Deposit {
        /// Sidechain to deposit to
        sidechain: Sidechain,
        /// Amount of BTC to deposit
        #[arg(value_parser = btc_amount_parser)]
        amount: bitcoin::Amount,
        /// Deposit fee in BTC
        #[arg(value_parser = btc_amount_parser)]
        fee: bitcoin::Amount,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config_path = match args.config_path {
        Some(config_path) => config_path,
        None => "./config.toml".into(),
    };
    let config: Config = confy::load_path(config_path)?;
    let node = Client::new(&config)?;
    match args.commands {
        Commands::Generate { number, amount } => {
            for hash in node.generate(number, amount).await? {
                println!("{}", hash);
            }
        }
        Commands::Getbalances => {
            let balances = node.get_balances().await?;
            println!("main balance:  {:>16}", format!("{}", balances.main));
            println!("zcash balance: {:>16}", format!("{}", balances.zcash));
        }
        Commands::Getnewaddress { chain } => {
            println!("{}", node.get_new_address(chain).await?);
        }
        Commands::Deposit {
            sidechain,
            amount,
            fee,
        } => {
            let txid = node.deposit(sidechain, amount, fee).await?;
            println!(
                "created deposit of {} to {} with fee {} and txid = {}",
                amount, sidechain, fee, txid
            );
        }
    }
    Ok(())
}

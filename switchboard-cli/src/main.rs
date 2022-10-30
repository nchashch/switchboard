use anyhow::Result;
use clap::{Parser, Subcommand};
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::types::ErrorObject;
use std::path::PathBuf;
use switchboard_api::{Chain, Sidechain, SidechainClient};
use switchboard_config::Config;
use switchboard_rpc::SwitchboardRpcClient;

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
    Hello {
        name: String,
    },
    /// Generate a mainchain block
    Generate {
        number: usize,
        #[arg(value_parser = btc_amount_parser)]
        amount: bitcoin::Amount,
    },
    Zcash {
        method: String,
        params: Option<Vec<String>>,
    },
    Main {
        method: String,
        params: Option<Vec<String>>,
    },
    /// Get balances for mainchain and all sidechains
    Getbalances,
    /// Get a new address
    Getnewaddress {
        chain: Chain,
    },
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
    let address = format!("http://{}", config.switchboard.socket_address()?);
    let client = HttpClientBuilder::default().build(address)?;
    let sidechain_client = SidechainClient::new(&config)?;
    match args.commands {
        Commands::Hello { name } => {
            let result = client.hello(name).await?;
            println!("{}", result);
        }
        Commands::Generate { number, amount } => {
            for hash in sidechain_client.generate(number, amount).await? {
                println!("{}", hash);
            }
        }
        Commands::Zcash { method, params } => {
            let result = match sidechain_client.zcash_request(method, params).await {
                Ok(result) => format!("{:#}", result),
                Err(jsonrpsee::core::Error::Call(err)) => {
                    ErrorObject::from(err).message().to_string()
                }
                Err(err) => format!("{}", err),
            };
            println!("{}", result);
        }
        Commands::Main { method, params } => {
            let result = match sidechain_client.main_request(method, params).await {
                Ok(result) => format!("{:#}", result),
                Err(jsonrpsee::core::Error::Call(err)) => {
                    ErrorObject::from(err).message().to_string()
                }
                Err(err) => format!("{}", err),
            };
            println!("{}", result);
        }
        Commands::Getbalances => {
            let balances = sidechain_client.get_balances().await?;
            println!("main balance:  {:>24}", format!("{}", balances.main));
            println!("zcash balance: {:>24}", format!("{}", balances.zcash));
        }
        Commands::Getnewaddress { chain } => {
            println!("{}", sidechain_client.get_new_address(chain).await?);
        }
        Commands::Deposit {
            sidechain,
            amount,
            fee,
        } => {
            let txid = sidechain_client.deposit(sidechain, amount, fee).await?;
            println!(
                "created deposit of {} to {} with fee {} and txid = {}",
                amount, sidechain, fee, txid
            );
        }
    }
    Ok(())
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::types::ErrorObject;
use serde_json::Value;
use std::path::PathBuf;
use switchboard::{
    api::{Chain, Sidechain},
    config::Config,
    server::SwitchboardRpcClient,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    datadir: Option<PathBuf>,
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
    /// Call zcash RPC directly
    Zcash {
        method: String,
        params: Option<Vec<String>>,
    },
    /// Call mainchain RPC directly
    Main {
        method: String,
        params: Option<Vec<String>>,
    },
    /// Get balances for mainchain and all sidechains
    Getbalances,
    /// Get block counts for mainchain and all sidechains
    Getblockcounts,
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
    /// Withdraw funds from a sidechain
    Withdraw {
        /// Sidechain to withdraw from
        sidechain: Sidechain,
        /// Amount of BTC to withdraw
        #[arg(value_parser = btc_amount_parser)]
        amount: bitcoin::Amount,
        /// Withdrawal fee in BTC, determines withdrawal priority in the bundle
        #[arg(value_parser = btc_amount_parser)]
        fee: bitcoin::Amount,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let home_dir = dirs::home_dir().unwrap();
    let datadir = args
        .datadir
        .unwrap_or_else(|| home_dir.join(".switchboard"));
    let config: Config = confy::load_path(datadir.join("config.toml"))?;
    let address = format!("http://{}", config.switchboard.socket_address()?);
    let client = HttpClientBuilder::default().build(address)?;
    match args.commands {
        Commands::Generate { number, amount } => {
            for hash in client.generate(number, amount.to_sat()).await? {
                print!("{}", hash);
            }
        }
        Commands::Zcash { method, params } => {
            let result = match client.zcash(method.clone(), prepare_params(params)).await {
                Ok(result) => {
                    if method == "help" {
                        let help_string = format!("{}", result)
                            .replace("\\n", "\n")
                            .replace("\\\"", "\"");
                        let mut chars = help_string.chars();
                        chars.next();
                        chars.next_back();
                        chars.as_str().into()
                    } else {
                        format!("{:#}", result)
                    }
                }
                Err(jsonrpsee::core::Error::Call(err)) => {
                    ErrorObject::from(err).message().to_string()
                }
                Err(err) => format!("{}", err),
            };
            print!("{}", result);
        }
        Commands::Main { method, params } => {
            let result = match client.main(method.clone(), prepare_params(params)).await {
                Ok(result) => {
                    if method == "help" {
                        let help_string = format!("{}", result)
                            .replace("\\n", "\n")
                            .replace("\\\"", "\"");
                        let mut chars = help_string.chars();
                        chars.next();
                        chars.next_back();
                        chars.as_str().into()
                    } else {
                        format!("{:#}", result)
                    }
                }
                Err(jsonrpsee::core::Error::Call(err)) => {
                    ErrorObject::from(err).message().to_string()
                }
                Err(err) => format!("{}", err),
            };
            print!("{}", result);
        }
        Commands::Getbalances => {
            let balances = client.getbalances().await?;
            print!("{}", balances);
        }
        Commands::Getblockcounts => {
            let block_counts = client.getblockcounts().await?;
            print!("{}", block_counts);
        }
        Commands::Getnewaddress { chain } => {
            print!("{}", client.getnewaddress(chain).await?);
        }
        Commands::Deposit {
            sidechain,
            amount,
            fee,
        } => {
            let txid = client
                .deposit(sidechain, amount.to_sat(), fee.to_sat())
                .await?;
            print!(
                "created deposit of {} to {} with fee {} and txid = {}",
                amount, sidechain, fee, txid
            );
        }
        Commands::Withdraw {
            sidechain,
            amount,
            fee,
        } => {
            client
                .withdraw(sidechain, amount.to_sat(), fee.to_sat())
                .await?;
            print!(
                "created withdrawal of {} from {} with fee {}",
                amount, sidechain, fee
            );
        }
    }
    Ok(())
}

fn prepare_params(params: Option<Vec<String>>) -> Option<Vec<Value>> {
    params.map(|p| {
        p.into_iter()
            .map(|param| match param.parse() {
                Ok(param) => param,
                Err(_) => Value::String(param),
            })
            .collect()
    })
}

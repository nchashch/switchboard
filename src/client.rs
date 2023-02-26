use crate::config::Config;
use anyhow::Result;
use ureq_jsonrpc::json;

#[derive(Clone)]
pub struct SidechainClient {
    main: ureq_jsonrpc::Client,
    zcash: ureq_jsonrpc::Client,
}

impl SidechainClient {
    pub fn new(config: &Config) -> Result<SidechainClient> {
        let main = ureq_jsonrpc::Client {
            host: "localhost".to_string(),
            port: config.main.port,
            user: config.switchboard.rpcuser.clone(),
            password: config.switchboard.rpcpassword.clone(),
            id: "switchboard".to_string(),
        };
        let zcash = ureq_jsonrpc::Client {
            host: "localhost".to_string(),
            port: config.zcash.port,
            user: config.switchboard.rpcuser.clone(),
            password: config.switchboard.rpcpassword.clone(),
            id: "switchboard".to_string(),
        };
        Ok(SidechainClient { main, zcash })
    }

    pub fn stop(&self) -> Result<Vec<String>, ureq_jsonrpc::Error> {
        let zcash = self.zcash.send_request("stop", &[]);
        let main = self.main.send_request("stop", &[]);
        Ok(vec![zcash?, main?])
    }

    /// This is used for setting up a new testing environment.
    pub fn activate_sidechains(&self) -> Result<(), ureq_jsonrpc::Error> {
        let active_sidechains = [(0, "zcash"), (1, "ethereum")];
        for (sidechain_number, sidechain_name) in active_sidechains {
            self.main.send_request::<ureq_jsonrpc::Value>(
                "createsidechainproposal",
                &[json!(sidechain_number), json!(sidechain_name)],
            )?;
        }
        self.main
            .send_request::<ureq_jsonrpc::Value>("generate", &[json!(200)])?;
        Ok(())
    }
}

// pub async fn refund(&self, sidechain: Sidechain, amount: u64, fee: u64) -> Result<()> {
//     match sidechain {
//         Sidechain::Zcash => {
//             let amount = Amount::from_sat(amount);
//             let fee = Amount::from_sat(fee);
//             ZcashClient::refund(&self.zcash, amount.into(), fee.into(), None, None).await?;
//         }
//         Sidechain::Ethereum => {
//             let mut unspent_withdrawals: Vec<(H256, U256)> =
//                 EthereumClient::get_unspent_withdrawals(&self.ethereum)
//                     .await?
//                     .iter()
//                     .map(|(id, uw)| (*id, uw.amount))
//                     .collect();
//             unspent_withdrawals.sort_by(|a, b| a.cmp(b));
//             let mut wei_amount: U256 = amount.into();
//             wei_amount *= SATOSHI;
//             let mut total_refund = U256::zero();
//             let mut refunded_withdrawals = HashSet::new();
//             dbg!(&unspent_withdrawals);
//             for (id, refund) in unspent_withdrawals.iter() {
//                 if total_refund > wei_amount {
//                     break;
//                 }
//                 total_refund += *refund;
//                 refunded_withdrawals.insert(id);
//             }
//             if total_refund < wei_amount {
//                 return Err(anyhow::Error::msg(
//                     "not enough funds in unspent withdrawals to refund",
//                 ));
//             }
//             let wei_change = total_refund - wei_amount;
//             for id in refunded_withdrawals.iter() {
//                 dbg!(id);
//                 EthereumClient::refund(&self.ethereum, id).await?;
//             }
//             let account = self.get_ethereum_account().await?;
//             let change: U256 = wei_change / SATOSHI;
//             let fee: U256 = fee.into();
//             // FIXME: Handle dust here.
//             if change > U256::zero() {
//                 EthereumClient::withdraw(&self.ethereum, &account, &change, &fee).await?;
//             }
//         }
//     };
//     Ok(())
// }

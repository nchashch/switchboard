use jsonrpsee::core::async_trait;
use jsonrpsee::http_server::HttpServerBuilder;
use jsonrpsee::proc_macros::rpc;

#[rpc(server)]
pub trait Rpc {
    #[method(name = "generate")]
    async fn generate(
        &self,
        number: usize,
        amount: zcash_rpc::AmountBtc,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error>;
}

#[async_trait]
impl RpcServer for Node {
    async fn generate(
        &self,
        number: usize,
        amount: zcash_rpc::AmountBtc,
    ) -> Result<Vec<bitcoin::BlockHash>, jsonrpsee::core::Error> {
        self.generate(number, amount.0).await
    }
}

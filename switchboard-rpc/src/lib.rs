use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;

pub struct Switchboardd;

#[rpc(server, client)]
pub trait SwitchboardRpc {
    /// A method for testing the switchboard RPC server
    #[method(name = "hello")]
    async fn hello(&self, name: String) -> Result<String, jsonrpsee::core::Error>;
}

#[async_trait]
impl SwitchboardRpcServer for Switchboardd {
    async fn hello(&self, name: String) -> Result<String, jsonrpsee::core::Error> {
        Ok(format!("Hello {}!", name))
    }
}

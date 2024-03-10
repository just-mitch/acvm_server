use std::net::SocketAddr;

use acvm_cli::execute_program_from_witness;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::types::ErrorObjectOwned;

#[rpc(server)]
pub trait Rpc {
    /// Normal method call example.
    #[method(name = "add")]
    fn add(&self, a: u16, b: u16) -> Result<u16, ErrorObjectOwned>;
}

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer for RpcServerImpl {
    fn add(&self, first_param: u16, second_param: u16) -> Result<u16, ErrorObjectOwned> {
        Ok(first_param + second_param)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    let _ = run_server().await?;

    futures::future::pending().await
}

async fn run_server() -> anyhow::Result<SocketAddr> {
    let server = Server::builder().build("127.0.0.1:9997").await?;

    let addr = server.local_addr()?;
    let handle = server.start(RpcServerImpl.into_rpc());

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(handle.stopped());

    Ok(addr)
}

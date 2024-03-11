#![warn(dead_code)]
use std::net::SocketAddr;

use acvm_cli::cli::execute_cmd::ExecuteCommand;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};

#[rpc(server)]
pub trait Rpc {
    /// Normal method call example.
    #[method(name = "run")]
    fn run(
        &self,
        output_witness: String,
        input_witness: String,
        bytecode: String,
        working_directory: String,
    ) -> Result<String, ErrorObjectOwned>;
}

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer for RpcServerImpl {
    fn run(
        &self,
        output_witness: String,
        input_witness: String,
        bytecode: String,
        working_directory: String,
    ) -> Result<String, ErrorObjectOwned> {
        let cmd_struct = ExecuteCommand {
            output_witness: Some(output_witness),
            input_witness,
            bytecode,
            working_directory,
            print: false,
        };
        acvm_cli::cli::execute_cmd::run_command(cmd_struct)
            .map_err(|_| ErrorObjectOwned::from(ErrorCode::InternalError))
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

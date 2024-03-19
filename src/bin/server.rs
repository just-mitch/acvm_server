#![warn(dead_code)]

use std::io::Read;
use std::net::SocketAddr;

use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};

use acir::circuit::Circuit;
use acir::native_types::WitnessMap;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::ops::{execute_circuit, DefaultForeignCallExecutor};

use acvm_server::errors::CliError;

pub(crate) fn execute_program_from_witness(
    inputs_map: &WitnessMap,
    bytecode: &[u8],
    foreign_call_resolver_url: Option<&str>,
) -> Result<WitnessMap, CliError> {
    let blackbox_solver = Bn254BlackBoxSolver::new();
    let circuit: Circuit = Circuit::deserialize_circuit(bytecode)
        .map_err(|_| CliError::CircuitDeserializationError())?;
    execute_circuit(
        &circuit,
        inputs_map.clone(),
        &blackbox_solver,
        &mut DefaultForeignCallExecutor::new(true, foreign_call_resolver_url),
    )
    .map_err(CliError::CircuitExecutionError)
}

#[rpc(server)]
pub trait Rpc {
    /// Normal method call example.
    #[method(name = "run")]
    fn run(&self, witness_map: Vec<u8>, bytecode: Vec<u8>) -> Result<WitnessMap, ErrorObjectOwned>;
}

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer for RpcServerImpl {
    fn run(&self, witness: Vec<u8>, bytecode: Vec<u8>) -> Result<WitnessMap, ErrorObjectOwned> {
        let witness_map = WitnessMap::try_from(witness.as_slice())
            .map_err(|_| ErrorObjectOwned::from(ErrorCode::InternalError))?;
        execute_program_from_witness(&witness_map, bytecode.as_slice(), None)
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

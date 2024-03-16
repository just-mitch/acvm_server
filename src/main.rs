#![warn(dead_code)]
use std::net::SocketAddr;
use std::path::PathBuf;

use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::Server;
use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};

use acir::circuit::Circuit;
use acir::native_types::WitnessMap;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::ops::{execute_circuit, DefaultForeignCallExecutor};
use nargo::NargoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilesystemError {
    #[error(
        " Error: cannot find {0} in expected location {1:?}.\n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),
    #[error(" Error: failed to parse toml file {0}.")]
    InvalidTomlFile(String),
    #[error(
      " Error: cannot find {0} in expected location {1:?}.\n Please generate this file at the expected location."
    )]
    MissingBytecodeFile(String, PathBuf),

    #[error(" Error: failed to read bytecode file {0}.")]
    InvalidBytecodeFile(String),

    #[error(" Error: failed to create output witness file {0}.")]
    OutputWitnessCreationFailed(String),

    #[error(" Error: failed to write output witness file {0}.")]
    OutputWitnessWriteFailed(String),
}

#[derive(Debug, Error)]
pub(crate) enum CliError {
    /// Filesystem errors
    #[error(transparent)]
    FilesystemError(#[from] FilesystemError),

    /// Error related to circuit deserialization
    #[error("Error: failed to deserialize circuit")]
    CircuitDeserializationError(),

    /// Error related to circuit execution
    #[error(transparent)]
    CircuitExecutionError(#[from] NargoError),

    /// Input Witness Value Error
    #[error("Error: failed to parse witness value {0}")]
    WitnessValueError(String),

    /// Input Witness Index Error
    #[error("Error: failed to parse witness index {0}")]
    WitnessIndexError(String),

    #[error(" Error: failed to serialize output witness.")]
    OutputWitnessSerializationFailed(),
}

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
    fn run(&self, witness_map: String, bytecode: String) -> Result<WitnessMap, ErrorObjectOwned>;
}

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer for RpcServerImpl {
    fn run(&self, witness: String, bytecode: String) -> Result<WitnessMap, ErrorObjectOwned> {
        let witness_map = WitnessMap::try_from(witness.as_bytes())
            .map_err(|_| ErrorObjectOwned::from(ErrorCode::InternalError))?;
        execute_program_from_witness(&witness_map, bytecode.as_bytes(), None)
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

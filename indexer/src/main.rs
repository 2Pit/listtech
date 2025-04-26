pub mod api;
pub mod app;
pub mod domain;
pub mod infra;

use anyhow::{Error, Result};
use app::swagger_server;
use corelib::telemetry::init::{init_logging, read_env_var};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    init_logging();

    // let grpc_port = read_env_var("INDEXER_GRPC_PORT", None)?;
    let swagger_port = read_env_var("INDEXER_SWAGGER_PORT", None)?;

    let swagger = tokio::spawn(swagger_server::run_swagger_server(swagger_port));
    // let grpc_api = tokio::spawn(grpc_server::run_grpc_server(grpc_port));

    let (swagger_res,) = tokio::try_join!(swagger)?;
    swagger_res?;

    Ok(())
}

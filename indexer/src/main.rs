pub mod api;
pub mod app;
pub mod domain;
pub mod infra;

use anyhow::{Error, Result};
use app::grpc_server;
use app::swagger_server;
use corelib::telemetry::init::{init_logging, read_u16_env};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    init_logging();

    let grpc_port = read_u16_env("INDEXER_GRPC_PORT", None)?;
    let swagger_port = read_u16_env("INDEXER_SWAGGER_PORT", None)?;

    let swagger = tokio::spawn(swagger_server::run_swagger_server(swagger_port));
    let grpc_api = tokio::spawn(grpc_server::run_grpc_server(grpc_port));

    let (swagger_res, grpc_res) = tokio::try_join!(swagger, grpc_api)?;
    swagger_res?;
    grpc_res?;

    Ok(())
}

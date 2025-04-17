mod api;
mod app;
mod domain;
mod infra;

use anyhow::Result;
use app::grpc_server::run_grpc_server;
use app::swagger_server::run_swagger_server;
use corelib::telemetry::init::{init_logging, read_u16_env};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let grpc_port = read_u16_env("SEARCHER_GRPC_PORT", None)?;
    let swagger_port = read_u16_env("SEARCHER_SWAGGER_PORT", None)?;

    let grpc_task = tokio::spawn(run_grpc_server(grpc_port));
    let swagger_task = tokio::spawn(run_swagger_server(swagger_port));

    let (grpc_res, swagger_res) = tokio::try_join!(grpc_task, swagger_task)?;
    grpc_res?;
    swagger_res?;

    Ok(())
}

mod api;
mod app;
mod domain;
mod infra;

use anyhow::Result;
use app::search_server::run_search_server;
use app::swagger_server::run_swagger_server;
use corelib::telemetry::init::{init_logging, read_env_var};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let index_dir = read_env_var("SEARCHER_INDEX_DIR", None)?;
    let grpc_port = read_env_var("SEARCHER_GRPC_PORT", None)?;
    let swagger_port = read_env_var("SEARCHER_SWAGGER_PORT", None)?;

    let grpc_task = tokio::spawn(run_search_server(grpc_port, index_dir));
    let swagger_task = tokio::spawn(run_swagger_server(swagger_port));

    let (grpc_res, swagger_res) = tokio::try_join!(grpc_task, swagger_task)?;
    grpc_res?;
    swagger_res?;

    Ok(())
}

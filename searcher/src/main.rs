mod api;
mod app;
mod domain;
mod infra;
mod model;

use crate::app::{api_server, swagger_server};
use anyhow::Result;
use corelib::telemetry::init::{init_logging, read_env_var};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let index_registry_dir: String = read_env_var("SEARCHER_INDEX_REGISTRY_DIR", None)?;
    let http_port = read_env_var("SEARCHER_HTTP_PORT", None)?;
    let swagger_port = read_env_var("SEARCHER_SWAGGER_PORT", None)?;

    if !std::path::Path::new(&index_registry_dir).exists() {
        tracing::error!("Index directory '{}' does not exist", index_registry_dir);
        std::process::exit(1);
    }

    let http_task = api_server::run_http_server(http_port, index_registry_dir);
    let swagger_task = swagger_server::run_swagger_server(swagger_port);

    tokio::try_join!(http_task, swagger_task)?;
    Ok(())
}

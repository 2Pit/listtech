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

    let index_dir: String = read_env_var("SEARCHER_INDEX_DIR", None)?;
    let http_port = read_env_var("SEARCHER_HTTP_PORT", None)?;
    let swagger_port = read_env_var("SEARCHER_SWAGGER_PORT", None)?;

    if !std::path::Path::new(&index_dir).exists() {
        tracing::error!("Index directory '{}' does not exist", index_dir);
        std::process::exit(1);
    }

    let http_task = tokio::spawn(api_server::run_http_server(http_port, index_dir));
    let swagger_task = tokio::spawn(swagger_server::run_swagger_server(swagger_port));

    let (http_res, swagger_res) = tokio::try_join!(http_task, swagger_task)?;
    http_res?;
    swagger_res?;

    Ok(())
}

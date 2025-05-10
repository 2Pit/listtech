pub mod api;
pub mod app;
pub mod infra;
pub mod model;

use anyhow::{Error, Result};
use app::api_server;
use corelib::telemetry::init::{init_logging, read_env_var};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    init_logging();

    let api_port = read_env_var("INDEXER_HTTP_PORT", None)?;
    let index_registry_dir = read_env_var("INDEXER_INDEX_REGISRY_DIR", None)?;

    let http_api = api_server::run_http_server(api_port, index_registry_dir);
    tokio::try_join!(http_api)?;

    Ok(())
}

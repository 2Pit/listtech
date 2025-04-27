pub mod api;
pub mod app;
pub mod infra;
pub mod model;

use anyhow::{Error, Result};
use app::{api_server, swagger_server};
use corelib::telemetry::init::{init_logging, read_env_var};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    init_logging();

    let api_port = read_env_var("INDEXER_HTTP_API_PORT", None)?;
    let swagger_port = read_env_var("INDEXER_SWAGGER_PORT", None)?;

    let swagger = tokio::spawn(swagger_server::run_swagger_server(swagger_port));
    let http_api = tokio::spawn(api_server::run_http_server(api_port));

    let (swagger_res, api_res) = tokio::try_join!(swagger, http_api)?;
    swagger_res?;
    api_res?;

    Ok(())
}

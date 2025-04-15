mod api;
mod indexing;

use anyhow::{Context, Error, Result};
use api::grpc_server;
use api::swagger;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    init_logging();

    let grpc_port: u16 = std::env::var("GRPC_PORT")
        .context("GRPC_PORT not set")?
        .parse()
        .context("GRPC_PORT should be a valid u16 number")?;

    let swagger_port = std::env::var("SWAGGER_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    let swagger = tokio::spawn(swagger::run_swagger_server(swagger_port));
    let grpc_api = tokio::spawn(grpc_server::run_grpc_server(grpc_port));

    let (swagger_res, grpc_res) = tokio::try_join!(swagger, grpc_api)?;
    swagger_res?;
    grpc_res?;

    Ok(())
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap())
        .with_file(true)
        .with_line_number(true)
        .init();
}

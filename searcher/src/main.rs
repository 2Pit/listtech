mod api;
mod app;
mod index;

use anyhow::{Context, Result};
use app::grpc_server::run_grpc_server;
use app::swagger_server::run_swagger_server;

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

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_file(true)
        .with_line_number(true)
        .init();
}

fn read_u16_env(var: &str, default: Option<u16>) -> Result<u16> {
    match std::env::var(var) {
        Ok(val) => val
            .parse()
            .with_context(|| format!("{} must be a valid u16 number", var)),
        Err(std::env::VarError::NotPresent) => {
            default.ok_or_else(|| anyhow::anyhow!("{} not set and no default provided", var))
        }
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

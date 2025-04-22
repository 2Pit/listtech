mod client;
mod table;

use anyhow::Result;
use clap::Parser;
use client::create_client;
use corelib::proto::searcher::SearchRequest;
use table::print_results;

use corelib::telemetry::init::{init_logging, read_env_var};

#[derive(Parser)]
struct Cli {
    /// Текст запроса
    #[arg(long)]
    query: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let port: u16 = read_env_var("SEARCHER_GRPC_PORT", None)?;

    let addr = format!("http://localhost:{}", port);
    let mut client = create_client(&addr).await?;

    let response = client
        .search(tonic::Request::new(SearchRequest {
            query: Cli::parse().query,
        }))
        .await?
        .into_inner();

    print_results(&response)?;
    Ok(())
}

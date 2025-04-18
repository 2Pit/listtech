mod client;
mod display;

use anyhow::Result;
use clap::Parser;
use client::create_client;
use display::print_hits_table;
use searcher::api::proto::searcher::SearchRequest;

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
            // filters: vec![],
            // sort_by: vec![],
            // offset: 0,
            // limit: 10,
        }))
        .await?
        .into_inner();

    print_hits_table(&response.hits)?;
    Ok(())
}

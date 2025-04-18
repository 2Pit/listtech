use crate::infra::index::SearchIndex;
use corelib::proto::searcher::search_service_server::SearchServiceServer;

use anyhow::{Context, Result};
use std::path::Path;
use tonic::transport::Server;

pub async fn run_search_server(port: u16, index_dir: String) -> Result<()> {
    let addr = format!("0.0.0.0:{port}")
        .parse()
        .context("Failed to parse gRPC socket address")?;

    let search_index = SearchIndex::open_from_path(Path::new(&index_dir))?;

    tracing::info!(port = %port, %addr, "Starting gRPC server");

    Server::builder()
        .add_service(SearchServiceServer::new(search_index))
        .serve(addr)
        .await
        .context("gRPC server exited unexpectedly")?;

    Ok(())
}

use crate::api::grpc::IndexerGrpc;
use corelib::proto::indexer::indexer_api_server::IndexerApiServer;

use anyhow::{Context, Result};
use std::path::Path;
use tonic::transport::Server;

pub async fn run_grpc_server(port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{port}")
        .parse()
        .context("Failed to parse gRPC socket address")?;

    let indexer_grpc = IndexerGrpc::create_indexer_grpc(Path::new("data/index")).await?;

    tracing::info!(port = %port, %addr, "Starting gRPC server");

    Server::builder()
        .add_service(IndexerApiServer::new(indexer_grpc))
        .serve(addr)
        .await
        .context("gRPC server exited unexpectedly")?;

    Ok(())
}

use crate::api::grpc_api::IndexerGrpc;
use crate::api::proto::indexer::indexer_api_server::IndexerApiServer;
use anyhow::{Context, Result};
use tonic::transport::Server;

pub async fn run_grpc_server(port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{port}")
        .parse()
        .context("Failed to parse gRPC socket address")?;

    tracing::info!(port = %port, %addr, "Starting gRPC server");

    Server::builder()
        .add_service(IndexerApiServer::new(IndexerGrpc::default()))
        .serve(addr)
        .await
        .context("gRPC server exited unexpectedly")?;

    Ok(())
}

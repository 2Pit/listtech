use super::grpc_api;

use crate::api::proto::indexer::indexer_api_server::IndexerApiServer;
use grpc_api::IndexerGrpc;
use tonic::transport::Server;

pub async fn run_grpc_server() {
    let port = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(50051);

    let addr = format!("0.0.0.0:{port}").parse().unwrap();

    tracing::info!("gRPC listening on {}", addr);
    Server::builder()
        .add_service(IndexerApiServer::new(IndexerGrpc::default()))
        .serve(addr)
        .await
        .unwrap();
}

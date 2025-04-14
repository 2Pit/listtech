use super::grpc_api;

use crate::api::proto::indexer::indexer_api_server::IndexerApiServer;
use grpc_api::IndexerGrpc;
use tonic::transport::Server;

pub async fn run_grpc_server(port: u16) {
    let addr = format!("0.0.0.0:{port}").parse().unwrap();

    tracing::info!("gRPC listening on {}", addr);
    Server::builder()
        .add_service(IndexerApiServer::new(IndexerGrpc::default()))
        .serve(addr)
        .await
        .unwrap();
}

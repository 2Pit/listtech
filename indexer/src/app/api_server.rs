use crate::infra::index::IndexState;
use crate::model;
use anyhow::Error;
use axum::response::IntoResponse;
use axum::{Router, routing::post};
use serde_cbor;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::error;
use tracing::info;

pub async fn run_http_server(port: u16) -> Result<(), Error> {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("cannot bind to HTTP port");

    let app = Router::new()
        .route("/v1/doc", post(handle_add_document))
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app).await.unwrap();
    Ok(())
}

/// Обработчик ручки POST /v1/doc
pub async fn handle_add_document(
    state: Arc<IndexState>,
    req: model::AddDocumentRequest,
) -> impl IntoResponse {
    match state.add_document_safely(req.0.document).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            error!(?err, "Failed to index document");
            (StatusCode::BAD_REQUEST, format!("Failed to index: {err}")).into_response()
        }
    }
}

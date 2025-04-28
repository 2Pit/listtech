use crate::infra::index::IndexState;
use crate::model;

use anyhow::Result;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Router, routing::post};
use hyper::StatusCode;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Запуск HTTP API сервера
pub async fn run_http_server(port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");

    // Читаем индекс
    info!("Opening search index at './data/index'");
    let index_state = match IndexState::init_index("./data/index").await {
        Ok(index_state) => Arc::new(index_state),
        Err(e) => {
            error!(error = %e, "Failed to initialize index");
            return Err(e);
        }
    };

    // Биндим сокет
    info!("Binding to {addr}");
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        error!(error = %e, "Failed to bind TCP socket");
        e
    })?;

    info!("Starting HTTP server on {addr}");

    let app = Router::new()
        .route("/v1/doc", post(handle_add_document))
        .with_state(index_state)
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app).await.map_err(|e| {
        error!(error = %e, "HTTP server failed");
        e.into()
    })
}

/// Обработчик ручки POST /v1/doc
pub async fn handle_add_document(
    State(state): State<Arc<IndexState>>,
    req: model::req_mapper::AddDocumentRequest,
) -> impl IntoResponse {
    match state.add_document_safely(req.0.document).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            error!(?err, "Failed to index document");
            (StatusCode::BAD_REQUEST, format!("Failed to index: {err}")).into_response()
        }
    }
}

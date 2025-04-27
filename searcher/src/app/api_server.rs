use crate::api;
use crate::infra::index::SearchIndex;

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
};
use hyper::StatusCode;
use std::{path::Path, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Запуск HTTP API сервера
pub async fn run_http_server(port: u16, index_dir: String) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");

    let search_index = Arc::new(SearchIndex::open_from_path(Path::new(&index_dir))?);

    info!(port = %port, addr = %addr, "Starting HTTP server");

    let app = Router::new()
        .route("/v1/select", post(handle_search)) // исправили на правильный эндпоинт
        .with_state(search_index)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app)
        .await
        .context("HTTP server exited unexpectedly")
}

/// Обработчик запроса поиска
pub async fn handle_search(
    State(index): State<Arc<SearchIndex>>,
    Json(req): Json<api::SearchRequest>,
) -> impl IntoResponse {
    use crate::infra::search::{build_search_response, execute_search};

    let top_docs = match execute_search(&index, &req.filter) {
        Ok(top_docs) => top_docs,
        Err(err) => {
            error!(?err, "Search execution failed");
            return (
                StatusCode::BAD_REQUEST,
                format!("Search execution failed: {err}"),
            )
                .into_response();
        }
    };

    let projection: Vec<&str> = req.projections.iter().map(String::as_str).collect();

    let response = match build_search_response(&index, &top_docs, &projection) {
        Ok(response) => response,
        Err(err) => {
            error!(?err, "Failed to build search response");
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to build response: {err}"),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(response)).into_response()
}

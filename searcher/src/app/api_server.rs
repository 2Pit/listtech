use crate::{infra::index::SearchIndex, model::req_mapper};
use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
};
use hyper::StatusCode;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Запуск HTTP API сервера
pub async fn run_http_server(port: u16, index_dir: String) -> Result<()> {
    let index = SearchIndex::open_from_path(&index_dir)?;
    let search_index = Arc::new(index);

    let addr = format!("0.0.0.0:{port}");
    info!(port = %port, addr = %addr, "Starting HTTP server");

    let app = Router::new()
        .route("/v1/select", post(handle_search))
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
    req_mapper::SearchRequest(req): req_mapper::SearchRequest,
) -> impl IntoResponse {
    use crate::infra::search::{build_search_response, execute_search};

    let top_docs = match execute_search(&index, &req) {
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

    let response = match build_search_response(&index, &top_docs, &req.select) {
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

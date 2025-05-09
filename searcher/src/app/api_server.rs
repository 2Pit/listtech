use anyhow::{Context, Result};
use axum::{Router, extract::State, routing::post};
use corelib::model::{accept::Accept, typed_request::TypedRequest, typed_response::TypedResponse};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::infra::search;
use crate::{
    api,
    infra::index_registry::{self, IndexRegistry},
};

/// Запуск HTTP API сервера
pub async fn run_http_server(port: u16, index_registry_dir: String) -> Result<()> {
    let index_registry =
        index_registry::load_all_indexes(std::path::Path::new(&index_registry_dir)).await?;

    // let index = SearchIndex::open_from_path(&index_dir)?;
    // let search_index = Arc::new(index);

    let addr = format!("0.0.0.0:{port}");
    info!(port = %port, addr = %addr, "Starting HTTP server");

    let app = Router::new()
        .route("/v1/select", post(handle_search))
        .with_state(index_registry)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app)
        .await
        .context("HTTP server exited unexpectedly")
}

/// Обработчик запроса поиска
pub async fn handle_search(
    Accept(accept): Accept,
    State(registry): State<IndexRegistry>,
    TypedRequest(req): TypedRequest<api::SearchRequest>,
) -> TypedResponse<api::SearchResponse> {
    let index_name = &req.from;
    let Some(index) = registry.inner.get(index_name) else {
        return TypedResponse::not_found(format!("Unknown index_name: {}", index_name), accept);
    };

    let top_docs = match search::execute_search(&index, &req) {
        Ok(top_docs) => top_docs,
        Err(err) => {
            error!(?err, "Search execution failed");
            return TypedResponse::bad_request(
                "searching_failed",
                "Search execution failed: {err}",
                accept,
            );
        }
    };

    match search::build_search_response(&index, &top_docs, &req) {
        Ok(response) => TypedResponse::ok(response, accept),
        Err(err) => {
            error!(?err, "Failed to build search response");
            TypedResponse::internal_error(format!("Failed to build response: {err}"), accept)
        }
    }
}

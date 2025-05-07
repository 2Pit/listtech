use anyhow::Result;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use corelib::model::accept::Accept;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

use crate::api;
use crate::api::GetSchemaResponse;
use crate::infra::index::IndexState;
use crate::infra::index_registry::{self, IndexRegistry};
use crate::model::typed_request::TypedRequest;
use crate::model::typed_response::TypedResponse;

/// Запуск HTTP API сервера
pub async fn run_http_server(port: u16, index_registry_dir: String) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");

    // Читаем индекс
    info!("Opening search index at './data/index'");
    let index_registry =
        index_registry::load_all_indexes(std::path::Path::new(&index_registry_dir)).await?;

    // Биндим сокет
    info!("Binding to {addr}");
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        error!(error = %e, "Failed to bind TCP socket");
        e
    })?;

    info!("Starting HTTP server on {addr}");

    let app = Router::new()
        .route("/v1/doc", post(handle_add_document))
        .route("/v1/schema/{schema_name}", get(get_schema))
        .route("/v1/schema", post(create_new_schema))
        .with_state(index_registry)
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app).await.map_err(|e| {
        error!(error = %e, "HTTP server failed");
        e.into()
    })
}

/// Обработчик ручки POST /v1/doc
pub async fn handle_add_document(
    Accept(accept): Accept,
    State(registry): State<IndexRegistry>,
    TypedRequest(body): TypedRequest<api::AddDocumentRequest>,
) -> TypedResponse<()> {
    let index_name = &body.document.index_name;

    let Some(index_state) = registry.inner.get(index_name) else {
        return TypedResponse::not_found(format!("Unknown index_name: {}", index_name), accept);
    };

    match index_state.add_document_safely(body.document).await {
        Ok(_) => TypedResponse::ok((), accept),
        Err(err) => {
            error!(?err, "Failed to index document");
            TypedResponse::bad_request("index_failed", format!("{err}"), accept)
        }
    }
}

/// Обработчик ручки GET /v1/schema
pub async fn get_schema(
    Accept(accept): Accept,
    State(registry): State<IndexRegistry>,
    Path(schema_name): Path<String>,
) -> impl IntoResponse {
    let Some(index_state) = registry.inner.get(&schema_name) else {
        return TypedResponse::not_found(format!("Schema '{}' not found", schema_name), accept);
    };

    TypedResponse::ok(
        GetSchemaResponse {
            schema: index_state.schema.clone().into(),
        },
        accept,
    )
}

pub async fn create_new_schema(
    Accept(accept): Accept,
    State(registry): State<IndexRegistry>,
    TypedRequest(schema): TypedRequest<api::AddSchemaRequest>,
) -> impl IntoResponse {
    let schema_name = &schema.schema.name;

    if registry.inner.contains_key(schema_name) {
        warn!("Schema '{}' already exists", schema_name);
        return TypedResponse::bad_request(
            "schema_existed",
            format!("Schema '{}' already exists", schema_name),
            accept,
        );
    }

    info!("Creating new schema '{}'", schema_name);

    let index_state = match IndexState::init_index_state(&registry, &schema.schema).await {
        Ok(state) => Arc::new(state),
        Err(err) => {
            return TypedResponse::internal_error(
                format!("Failed to initialize index: {err}"),
                accept,
            );
        }
    };

    registry.inner.insert(schema_name.clone(), index_state);
    TypedResponse::created(api::AddSchemaResponse, accept)
}

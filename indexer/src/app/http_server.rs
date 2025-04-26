use crate::api::AddDocumentRequest;
use axum::{
    body::Body,
    extract::{FromRequest, Request},
    http::{StatusCode, header::CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};

const MAX_BODY_SIZE: usize = 5 * 1024 * 1024; // 5MB лимит

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAddDocumentRequest(pub AddDocumentRequest);

impl<S> FromRequest<S, Body> for HttpAddDocumentRequest
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
            .await
            .map_err(|e| ServerError::InvalidRequest(e.to_string()).into())?;

        if bytes.len() > MAX_BODY_SIZE {
            return Err(ServerError::InvalidRequest("Payload too large".into()).into());
        }

        match parts
            .headers
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
        {
            Some("application/json") => serde_json::from_slice(&bytes)
                .map_err(|e| ServerError::InvalidRequest(format!("Invalid JSON: {e}")).into()),
            Some("application/cbor") => serde_cbor::from_slice(&bytes)
                .map_err(|e| ServerError::InvalidRequest(format!("Invalid CBOR: {e}")).into()),
            Some(other) => Err(ServerError::InvalidRequest(format!(
                "Unsupported Content-Type: {other}"
            ))
            .into()),
            None => Err(ServerError::InvalidRequest("Missing Content-Type header".into()).into()),
        }
    }
}

#[derive(Debug)]
pub enum ServerError {
    InvalidRequest(String),
    InternalError,
}

impl Into<(StatusCode, String)> for ServerError {
    fn into(self) -> (StatusCode, String) {
        match self {
            ServerError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ServerError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into())
            }
        }
    }
}

use crate::infra::index::IndexState;
use axum::response::IntoResponse;
use serde_cbor;
use std::sync::Arc;
use tracing::error;

/// Обработчик ручки POST /v1/doc
pub async fn handle_add_document(
    state: Arc<IndexState>,
    req: HttpAddDocumentRequest,
) -> impl IntoResponse {
    match state.add_document_safely(req.0.document).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            error!(?err, "Failed to index document");
            (StatusCode::BAD_REQUEST, format!("Failed to index: {err}")).into_response()
        }
    }
}

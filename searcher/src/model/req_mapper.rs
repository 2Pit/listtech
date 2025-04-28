use crate::api;
use axum::{
    body::Body,
    extract::{FromRequest, Request},
    http::{StatusCode, header::CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};

const MAX_BODY_SIZE: usize = 5 * 1024 * 1024; // 5MB лимит

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest(pub api::SearchRequest);

impl<S> FromRequest<S, Body> for SearchRequest
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
    // InternalError,
}

impl Into<(StatusCode, String)> for ServerError {
    fn into(self) -> (StatusCode, String) {
        match self {
            ServerError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            // ServerError::InternalError => {
            // (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into())
            // }
        }
    }
}

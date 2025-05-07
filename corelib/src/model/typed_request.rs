use anyhow::Error;
use axum::{
    body::Body,
    extract::{FromRequest, FromRequestParts, Request},
    http::header::CONTENT_TYPE,
};
use http::{StatusCode, request::Parts};
use serde::de::DeserializeOwned;

use super::typed_response::ErrorResponse;

const MAX_BODY_SIZE: usize = 5 * 1024 * 1024; // 5MB лимит

/// Универсальный десериализуемый запрос (JSON или CBOR)
pub struct TypedRequest<T>(pub T);

impl<T> TypedRequest<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<S, T> FromRequest<S, Body> for TypedRequest<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let headers = parts.headers.clone();

        let bytes = axum::body::to_bytes(body, MAX_BODY_SIZE)
            .await
            .map_err(|e| {
                ErrorResponse::bad_request("invalid_body", format!("Failed to read body: {e}"))
            })?;

        if bytes.len() > MAX_BODY_SIZE {
            return Err(ErrorResponse::bad_request("too_large", "Payload too large"));
        }

        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        let parsed = match content_type {
            "application/json" => serde_json::from_slice(&bytes).map_err(Error::from),
            "application/cbor" => serde_cbor::from_slice(&bytes).map_err(Error::from),
            other => {
                return Err(ErrorResponse::bad_request(
                    "unsupported_content_type",
                    format!("Unsupported Content-Type: {other}"),
                ));
            }
        };

        parsed
            .map(TypedRequest)
            .map_err(|e| ErrorResponse::bad_request("deserialization_error", e.to_string()))
    }
}

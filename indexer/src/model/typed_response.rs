use axum::body::Body;
use axum::response::IntoResponse;
use bytes::Bytes;
use http::{Response, StatusCode};
use serde::Serialize;

/// Универсальный ответ, сериализуемый в JSON или CBOR в зависимости от Accept
pub struct TypedResponse<T> {
    pub value: T,
    pub accept: Option<String>,
}

impl<T: Serialize> IntoResponse for TypedResponse<T> {
    fn into_response(self) -> Response<Body> {
        match self.accept.as_deref() {
            Some("application/cbor") => match serde_cbor::to_vec(&self.value) {
                Ok(bytes) => to_response(bytes, "application/cbor"),
                Err(err) => error_response(format!("CBOR serialization error: {err}")),
            },
            _ => match serde_json::to_vec(&self.value) {
                Ok(bytes) => to_response(bytes, "application/json"),
                Err(err) => error_response(format!("JSON serialization error: {err}")),
            },
        }
    }
}

fn to_response(body: Vec<u8>, content_type: &str) -> Response<Body> {
    let body = Body::from(Bytes::from(body)); // Body = UnsyncBoxBody<Bytes, axum::Error>
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .body(body)
        .unwrap()
}

fn error_response(message: String) -> Response<Body> {
    let body = Body::from(Bytes::from(message));
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(body)
        .unwrap()
}

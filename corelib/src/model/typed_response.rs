use anyhow::Error;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
use serde::Serialize;

pub type TypedResponse<T> = TypedResponseResult<T, ErrorResponse>;

/// Обёртка над `Result<T, E>`, сериализуемая в JSON или CBOR
pub struct TypedResponseResult<T, E> {
    pub result: Result<T, E>,
    pub accept: Option<String>,
}

impl<T, E> TypedResponseResult<T, E> {
    pub fn new(result: Result<T, E>, accept: Option<String>) -> Self {
        Self { result, accept }
    }
}

impl<T, E> IntoResponse for TypedResponseResult<T, E>
where
    T: Serialize,
    E: Into<ErrorResponse>,
{
    fn into_response(self) -> Response<Body> {
        match self.result {
            Ok(value) => serialize_payload(&value, self.accept, StatusCode::OK),
            Err(err) => {
                let err = err.into();
                serialize_payload(&err, self.accept, err.status)
            }
        }
    }
}

impl<T> TypedResponse<T> {
    pub fn ok(value: T, accept: Option<String>) -> Self {
        Self {
            result: Ok(value),
            accept,
        }
    }

    pub fn created(value: T, accept: Option<String>) -> Self {
        Self {
            result: Ok(value),
            accept,
        }
    }

    pub fn bad_request(code: &str, message: impl Into<String>, accept: Option<String>) -> Self {
        Self::new(Err(ErrorResponse::bad_request(code, message)), accept)
    }

    pub fn not_found(message: impl Into<String>, accept: Option<String>) -> Self {
        Self::new(Err(ErrorResponse::not_found(message)), accept)
    }

    pub fn conflict(code: &str, message: impl Into<String>, accept: Option<String>) -> Self {
        Self::new(Err(ErrorResponse::conflict(code, message)), accept)
    }

    pub fn internal_error(message: impl Into<String>, accept: Option<String>) -> Self {
        Self::new(Err(ErrorResponse::internal_error(message)), accept)
    }
}

/// Ошибка, возвращаемая API. Не сериализуется статус.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,

    #[serde(skip_serializing)]
    pub status: StatusCode,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response<Body> {
        serialize_payload(&self, None, self.status)
    }
}

impl ErrorResponse {
    pub fn new(code: &str, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            status,
        }
    }

    pub fn bad_request(code: &str, message: impl Into<String>) -> Self {
        Self::new(code, message, StatusCode::BAD_REQUEST)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message, StatusCode::NOT_FOUND)
    }

    pub fn conflict(code: &str, message: impl Into<String>) -> Self {
        Self::new(code, message, StatusCode::CONFLICT)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message, StatusCode::INTERNAL_SERVER_ERROR)
    }
}

fn serialize_payload<T: Serialize>(
    value: &T,
    accept: Option<String>,
    status: StatusCode,
) -> Response<Body> {
    let mime = accept.as_deref();
    let body_result = match mime {
        Some("application/cbor") => serde_cbor::to_vec(value).map_err(Error::from),
        _ => serde_json::to_vec(value).map_err(Error::from),
    };

    match body_result {
        Ok(body) => {
            let content_type = match mime {
                Some("application/cbor") => "application/cbor",
                _ => "application/json",
            };
            Response::builder()
                .status(status)
                .header("Content-Type", content_type)
                .body(Body::from(Bytes::from(body)))
                .unwrap()
        }
        Err(e) => {
            let fallback = format!("{{\"code\":\"serialization_error\",\"message\":\"{}\"}}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(fallback))
                .unwrap()
        }
    }
}

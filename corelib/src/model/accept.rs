use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[derive(Debug, Clone)]
pub struct Accept(pub Option<String>);

impl<S> FromRequestParts<S> for Accept
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let accept = parts
            .headers
            .get("accept")
            .and_then(|h| h.to_str().ok())
            .map(str::to_string);
        Ok(Accept(accept))
    }
}

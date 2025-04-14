use axum::extract::Path;
use axum::response::{Html, IntoResponse, Response};
use include_dir::{include_dir, Dir};

static SWAGGER_UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static/swagger-ui");

pub async fn serve_indexer_html() -> Response {
    match SWAGGER_UI_DIR.get_file("indexer_index.html") {
        Some(file) => Html(file.contents()).into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            "Indexer Swagger not found",
        )
            .into_response(),
    }
}

pub async fn serve_searcher_html() -> Response {
    match SWAGGER_UI_DIR.get_file("searcher_index.html") {
        Some(file) => Html(file.contents()).into_response(),
        None => (
            axum::http::StatusCode::NOT_FOUND,
            "Searcher Swagger not found",
        )
            .into_response(),
    }
}

pub async fn serve_static(Path(path): Path<String>) -> Response {
    let file_path = path.trim_start_matches('/');
    match SWAGGER_UI_DIR.get_file(file_path) {
        Some(file) => {
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                file.contents(),
            )
                .into_response()
        }
        None => (
            axum::http::StatusCode::NOT_FOUND,
            format!("Static file not found: {}", file_path),
        )
            .into_response(),
    }
}

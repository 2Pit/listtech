use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};
use mime_guess::mime;
use tracing::error;

static SWAGGER_UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static/swagger-ui");

pub async fn serve_swagger_ui(Path(path): Path<String>) -> Response {
    let file_path = path.trim_start_matches('/');

    match SWAGGER_UI_DIR.get_file(file_path) {
        Some(file) => {
            let mime = mime_guess::from_path(file_path).first_or(mime::APPLICATION_OCTET_STREAM);
            ([(header::CONTENT_TYPE, mime.as_ref())], file.contents()).into_response()
        }
        None => {
            error!("Static file not found: {file_path}");
            (StatusCode::NOT_FOUND, "Not Found").into_response()
        }
    }
}

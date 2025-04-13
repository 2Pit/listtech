use axum::Json;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use include_dir::{include_dir, Dir};
use serde_json::Value;
use std::{env, fs};

static SWAGGER_UI_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static/swagger-ui");

pub async fn serve_swagger_ui(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let file = SWAGGER_UI_DIR
        .get_file(path)
        .or_else(|| SWAGGER_UI_DIR.get_file("index.html"));

    match file {
        Some(f) => Html(f.contents()).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

pub async fn serve_swagger_json() -> impl IntoResponse {
    let json_path = format!(
        "{}/proto/api.swagger.json",
        env::var("OUT_DIR").unwrap_or_default()
    );

    match fs::read_to_string(&json_path) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(json) => Json(json).into_response(),
            Err(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid JSON",
            )
                .into_response(),
        },
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Swagger JSON not found",
        )
            .into_response(),
    }
}

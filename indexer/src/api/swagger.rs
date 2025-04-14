use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use corelib::swagger::{serve_indexer_html, serve_static};

pub async fn run_swagger_server() {
    let port = std::env::var("INDEXER_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("cannot bind to HTTP port");

    let app = Router::new()
        .route("/swagger/", get(serve_indexer_html))
        .route("/swagger-ui/{*path}", get(serve_static))
        .layer(TraceLayer::new_for_http());

    info!("HTTP server running at http://localhost:{port}/swagger/");
    axum::serve(listener, app).await.unwrap();
}

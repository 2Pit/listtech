use axum::{routing::get, Router};
use core::swagger::serve_swagger_ui;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init_logging();

    let port = std::env::var("INDEXER_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("cannot bind to port");

    let app = Router::new()
        .route("/swagger/{*path}", get(serve_swagger_ui))
        .layer(TraceLayer::new_for_http());

    info!("Running indexer at http://localhost:{port}/swagger/index.html");

    axum::serve(listener, app).await.unwrap();
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap())
        .with_file(true)
        .with_line_number(true)
        .init();
}

use axum::{routing::get, Router};
use core::swagger::{serve_swagger_json, serve_swagger_ui};
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
        .route("/swagger/swagger.json", get(serve_swagger_json))
        .layer(TraceLayer::new_for_http());

    info!("Running indexer at http://localhost:{}/swagger/", port);

    axum::serve(listener, app).await.unwrap();
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true) // если хочешь показывать путь к файлу
        .with_line_number(true) // можно и строку
        .init();
}

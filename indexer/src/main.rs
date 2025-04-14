mod api;

use api::grpc_server;
use api::swagger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    init_logging();

    let grpc_port = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(50051);

    let http_server = tokio::spawn(swagger::run_swagger_server());
    let grpc_server = tokio::spawn(grpc_server::run_grpc_server(grpc_port));

    tokio::try_join!(http_server, grpc_server)?;

    Ok(())
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap())
        .with_file(true)
        .with_line_number(true)
        .init();
}

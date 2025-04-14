mod api;

use api::grpc_server;
use api::swagger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    init_logging();

    let swagger = tokio::spawn(swagger::run_swagger_server());
    let grpc_api = tokio::spawn(grpc_server::run_grpc_server());

    tokio::try_join!(swagger, grpc_api)?;

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

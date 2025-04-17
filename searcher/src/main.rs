mod index;

// use crate::index::SearchIndex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    // let index = SearchIndex::open_from_path("./data/index")?;
    println!("Index and reader successfully initialized.");

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

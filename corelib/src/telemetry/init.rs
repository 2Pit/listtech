use std::str::FromStr;

use anyhow::{Context, Result};

pub fn init_logging() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_file(true)
        .with_line_number(true)
        .init();
}

pub fn read_env_var<T>(key: &str, default: Option<T>) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static, // + 'static,
{
    match std::env::var(key) {
        Ok(val) => val
            .parse::<T>()
            .with_context(|| format!("{key} is invalid")),
        Err(std::env::VarError::NotPresent) => {
            default.ok_or_else(|| anyhow::anyhow!("{key} not set and no default provided"))
        }
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

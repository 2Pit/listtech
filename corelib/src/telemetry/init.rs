use anyhow::{Context, Result};

pub fn init_logging() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_file(true)
        .with_line_number(true)
        .init();
}

pub fn read_u16_env(var: &str, default: Option<u16>) -> Result<u16> {
    match std::env::var(var) {
        Ok(val) => val
            .parse()
            .with_context(|| format!("{} must be a valid u16 number", var)),
        Err(std::env::VarError::NotPresent) => {
            default.ok_or_else(|| anyhow::anyhow!("{} not set and no default provided", var))
        }
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

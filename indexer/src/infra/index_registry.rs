use anyhow::{Context, Result};
use dashmap::DashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use super::index::IndexState;

#[derive(Clone)]
pub struct IndexRegistry {
    pub inner: Arc<DashMap<String, Arc<IndexState>>>,
    pub indexes_root: PathBuf,
}

pub async fn load_all_indexes(repo_path: &Path) -> Result<IndexRegistry> {
    let registry = Arc::new(DashMap::new());

    let entries = fs::read_dir(repo_path).context("Failed to read index repository dir")?;

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let schema_name = entry.file_name().to_string_lossy().to_string();
        let schema_path = entry.path();
        let index_path = schema_path.join("index");

        match IndexState::read_index_state(&index_path, &schema_name).await {
            Ok(index_state) => {
                registry.insert(schema_name.clone(), Arc::new(index_state));
                tracing::info!(%schema_name, "Loaded index");
            }
            Err(e) => {
                tracing::warn!(%schema_name, error = ?e, "Failed to load index");
            }
        }
    }

    Ok(IndexRegistry {
        inner: registry,
        indexes_root: repo_path.to_path_buf(),
    })
}

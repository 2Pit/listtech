use anyhow::{Context, Result};
use dashmap::DashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::index::SearchIndex;

#[derive(Clone)]
pub struct IndexRegistry {
    pub inner: Arc<DashMap<String, Arc<SearchIndex>>>,
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

        match SearchIndex::open_from_path(index_path.to_str().unwrap()) {
            // match SearchIndex::open_from_path_to_ram(index_path.to_str().unwrap()) {
            Ok(search_index) => {
                registry.insert(schema_name.clone(), Arc::new(search_index));
                tracing::info!(%schema_name, "Loaded search index");
            }
            Err(e) => {
                tracing::error!(%schema_name, error = ?e, "Failed to load search index");
            }
        }
    }

    Ok(IndexRegistry {
        inner: registry,
        indexes_root: repo_path.to_path_buf(),
    })
}

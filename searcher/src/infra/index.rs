use anyhow::{Context, Result};
use std::path::Path;
use tantivy::{Index, IndexReader, ReloadPolicy};

pub struct SearchIndex {
    pub index: Index,
    pub reader: IndexReader,
}

impl SearchIndex {
    pub fn open_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let index = Index::open_in_dir(&path)
            .with_context(|| format!("Failed to open index in {:?}", path.as_ref()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create IndexReader")?;

        // let all_fields = index.schema().fields().map(|f| f.0).collect();

        Ok(Self { index, reader })
    }
}

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tantivy::schema::Schema;
use tantivy::{Index, IndexWriter};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct IndexState {
    pub writer: Arc<Mutex<IndexWriter>>,
    pub schema: Schema,
    pub index: Index,
}

impl IndexState {
    pub async fn init_index(path: &Path, schema: Schema) -> anyhow::Result<IndexState> {
        let index = if path.exists() {
            Index::open_in_dir(path)?
        } else {
            std::fs::create_dir_all(path)?;
            Index::create_in_dir(path, schema.clone())?
        };

        let writer = index.writer(50_000_000)?; // 50 MB
        let writer = Arc::new(Mutex::new(writer));

        // автокоммит
        {
            let writer_clone = writer.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    let mut w = writer_clone.lock().await;
                    if let Err(e) = w.commit() {
                        tracing::error!(error = %e, "Failed to autocommit index");
                    } else {
                        tracing::info!("Index autocommitted");
                    }
                }
            });
        }

        Ok(IndexState {
            writer,
            schema,
            index,
        })
    }
}

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tantivy::{Index, IndexWriter};
use tokio::sync::Mutex;

use super::schema::InnerSchema;

#[derive(Clone)]
pub struct IndexState {
    pub index: Index,
    pub schema: InnerSchema,
    pub writer: Arc<Mutex<IndexWriter>>,
}

impl IndexState {
    pub async fn init_index2(index_dir: &str) -> Result<IndexState> {
        let index: Index = Index::open_in_dir(Path::new(index_dir))?;
        let schema = InnerSchema::read_schema(&format!("{}/listtech_schema.json", index_dir))?;
        let writer = Self::init_writer(&index).await?;

        Ok(IndexState {
            index,
            schema,
            writer,
        })
    }

    async fn init_writer(index: &Index) -> Result<Arc<Mutex<IndexWriter>>> {
        let writer = index.writer(2_000_000_000)?; // 2 GB
        let writer = Arc::new(Mutex::new(writer));

        // автокоммит по таймеру
        {
            let writer_clone = writer.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    let mut w = writer_clone.lock().await;
                    if let Err(e) = w.commit() {
                        tracing::error!(error = %e, "Failed to autocommit index");
                    } else {
                        tracing::info!("Index autocommitted");
                    }
                }
            });
        }

        Ok(writer)
    }
}

use crate::api;
use crate::model::doc_mapper;

use anyhow::{Context, Result};
use corelib::model::delta_schema::DeltaSchema;
use corelib::model::meta_schema::MetaSchema;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tantivy::schema::Term;
use tantivy::{Index, IndexWriter};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct IndexState {
    pub index: Index,
    pub schema: MetaSchema,
    pub writer: Arc<Mutex<IndexWriter>>,
}

impl IndexState {
    pub async fn init_index(index_dir: &str) -> Result<IndexState> {
        let index: Index = Index::open_in_dir(Path::new(index_dir))?;
        let delta_schema =
            DeltaSchema::from_json_file(&format!("{}/delta_schema.json", index_dir))?;
        let meta_schema = MetaSchema::from_tantivy_and_delta(&index.schema(), delta_schema)?;
        let writer = Self::init_writer(&index).await?;

        Ok(IndexState {
            index,
            schema: meta_schema,
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

    pub async fn add_document_safely(&self, doc: api::Document) -> Result<()> {
        let tantivy_doc =
            doc_mapper::to_tantivy_doc(&self.schema, &doc).context("invalid document structure")?;

        let writer = self.writer.lock().await;

        let id_col_name = self.schema.id_column.name.clone();
        let term = doc
            .fields
            .into_iter()
            .find(|field| field.name == id_col_name)
            .ok_or_else(|| anyhow::anyhow!("ID not found"))
            .and_then(|field| field.value.ok_or_else(|| anyhow::anyhow!("ID is null")))
            .and_then(|id_value| match id_value {
                api::FieldValue::String(id) => Ok(Term::from_field_text(
                    self.schema.id_column.idx,
                    id.as_str(),
                )),
                api::FieldValue::Long(id) => {
                    Ok(Term::from_field_i64(self.schema.id_column.idx, id))
                }
                other => Err(anyhow::anyhow!("Unsupported ID type: {}", other)),
            })?;
        writer.delete_term(term);

        loop {
            match writer.add_document(tantivy_doc.clone()) {
                Ok(_) => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }
}

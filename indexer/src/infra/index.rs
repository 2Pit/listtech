use anyhow::{Context, Result};
use corelib::model::meta_schema::MetaSchema;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use tokio::sync::Mutex;

use crate::api;
use crate::model::doc_mapper;

#[derive(Clone)]
pub struct IndexState {
    pub index: Index,
    pub schema: MetaSchema,
    pub writer: Arc<Mutex<IndexWriter>>,
}

impl IndexState {
    pub async fn create_index_state(
        api_schema: api::MetaSchema,
        index_dir: &str,
    ) -> Result<IndexState> {
        let tantivy_schema = create_tantivy_schema_from_api(&api_schema);
        let index = Index::create_in_dir(Path::new(index_dir), tantivy_schema)?;
        let meta_schema = MetaSchema::from_api(&index.schema(), api_schema)?;
        let writer = Self::init_writer(&index).await?;

        Ok(IndexState {
            index,
            schema: meta_schema,
            writer,
        })
    }

    pub async fn read_index_state(index_dir: &str) -> Result<IndexState> {
        let index: Index = Index::open_in_dir(Path::new(index_dir))?;
        let delta_schema =
            api::MetaSchema::from_json_file(&format!("{}/delta_schema.json", index_dir))?;
        let meta_schema = MetaSchema::from_api(&index.schema(), delta_schema)?;
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

pub fn create_tantivy_schema_from_api(api_schema: &api::MetaSchema) -> tantivy::schema::Schema {
    let mut schema_builder = tantivy::schema::Schema::builder();

    api_schema.columns.iter().for_each(|api_col| {
        // let is_id = api_col.modifiers.contains(&api::MetaColumnModifier::Id);
        let is_eq = api_col.modifiers.contains(&api::MetaColumnModifier::Equals);
        let is_sort_range = api_col
            .modifiers
            .contains(&api::MetaColumnModifier::FastSortable);
        let is_full_text = api_col
            .modifiers
            .contains(&api::MetaColumnModifier::FullText);
        // let is_nullable = api_col
        // .modifiers
        // .contains(&api::MetaColumnModifier::Nullable);

        match api_col.column_type {
            api::MetaColumnType::Bool
            | api::MetaColumnType::Ulong
            | api::MetaColumnType::Long
            | api::MetaColumnType::Double => {
                let mut opt = NumericOptions::from(STORED);
                if is_eq {
                    opt = opt | NumericOptions::from(INDEXED)
                };
                if is_sort_range {
                    opt = opt | NumericOptions::from(FAST)
                };
                match api_col.column_type {
                    api::MetaColumnType::Bool => {
                        schema_builder.add_bool_field(&api_col.name, opt);
                    }
                    api::MetaColumnType::Ulong => {
                        schema_builder.add_u64_field(&api_col.name, opt);
                    }
                    api::MetaColumnType::Long => {
                        schema_builder.add_i64_field(&api_col.name, opt);
                    }
                    api::MetaColumnType::Double => {
                        schema_builder.add_f64_field(&api_col.name, opt);
                    }
                    _ => todo!(),
                }
            }
            api::MetaColumnType::DateTime => {
                let mut opt = DateOptions::from(STORED);
                if is_eq {
                    opt = opt | DateOptions::from(INDEXED)
                };
                if is_sort_range {
                    opt = opt | DateOptions::from(FAST)
                };
                schema_builder.add_date_field(&api_col.name, opt);
            }
            api::MetaColumnType::String => {
                let mut opt = TextOptions::from(STORED);
                if is_eq {
                    opt = opt | TextOptions::from(STRING)
                };
                if is_sort_range {
                    opt = opt | TextOptions::from(FAST)
                };
                if is_full_text {
                    opt = opt | TextOptions::from(TEXT)
                };
                schema_builder.add_text_field(&api_col.name, opt);
            }
            api::MetaColumnType::Bytes => {
                let mut opt = BytesOptions::from(STORED);
                if is_eq {
                    opt = opt | BytesOptions::from(INDEXED)
                };
                if is_sort_range {
                    opt = opt | BytesOptions::from(FAST)
                };
                schema_builder.add_bytes_field(&api_col.name, opt);
            }
            api::MetaColumnType::Tree => {
                let mut opt = FacetOptions::from(STORED);
                if is_eq {
                    opt = opt | FacetOptions::from(INDEXED)
                };
                // if let Some(tpe) = col_modif.1 {
                //     opt = opt | FacetOptions::from(tpe)
                // };
                schema_builder.add_facet_field(&api_col.name, opt);
            }
        }
    });
    return schema_builder.build();
}

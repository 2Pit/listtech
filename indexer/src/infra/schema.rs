use crate::api;
use crate::api::{ColumnType, FieldValue};
use anyhow::Result;
use anyhow::anyhow;
use chrono::{DateTime as CronoDateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::{fs, path::Path};
use tantivy::DateTime as TantivyDateTime;
use tantivy::TantivyError;
use tantivy::schema::Facet;
use tantivy::schema::Field as TanField;
use tantivy::schema::document::TantivyDocument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerSchema {
    pub name: String,
    pub version: u32,
    pub id_column: InnerColumnType,
    pub column_by_name: HashMap<String, InnerColumnType>,
    pub columns: Vec<InnerColumnType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerColumnType {
    pub tan_field: TanField,
    pub field_type: ColumnType,
    pub name: String,
    pub is_stored: bool,
    pub is_eq: bool,
    pub is_fast: bool,
    pub is_tree: bool,
    pub is_nullable: bool,
}

impl InnerSchema {
    pub fn read_schema(file_path: &str) -> Result<InnerSchema> {
        let path = Path::new(file_path);
        let contents = fs::read_to_string(path)?; // Читаем файл как строку
        let schema = serde_json::from_str::<InnerSchema>(&contents)?; // Парсим JSON в структуру Schema
        Ok(schema)
    }

    pub fn to_tantivy_doc(&self, doc: &api::Document) -> Result<TantivyDocument> {
        let mut compact_doc = TantivyDocument::new();

        for field in &doc.fields {
            let field_name = &field.name;
            let field_entry = self
                .column_by_name
                .get(field_name)
                .ok_or(anyhow!("Unknown filed: {}", field_name))?;
            let tan_field = field_entry.tan_field;

            if let Some(ref value) = field.value {
                use FieldValue::*;

                let field_type = &field_entry.field_type;

                match (value, field_type) {
                    (Bool(b), ColumnType::Bool) => compact_doc.add_bool(tan_field, *b),
                    (Long(i), ColumnType::Long) => compact_doc.add_i64(tan_field, *i),
                    (Ulong(u), ColumnType::Ulong) => compact_doc.add_u64(tan_field, *u),
                    (Double(f), ColumnType::Double) => compact_doc.add_f64(tan_field, *f),
                    (String(s), ColumnType::String) => compact_doc.add_text(tan_field, s),
                    (Bytes(b), ColumnType::Bytes) => compact_doc.add_bytes(tan_field, b.as_slice()),
                    (DateTime(iso_date), ColumnType::DateTime) => {
                        let dt: CronoDateTime<Utc> =
                            iso_date.parse().expect("Invalid ISO 8601 format");
                        compact_doc.add_date(
                            tan_field,
                            TantivyDateTime::from_timestamp_micros(dt.timestamp_micros()),
                        )
                    }
                    (Tree(paths), ColumnType::Tree) => {
                        for path in paths.into_iter() {
                            compact_doc.add_facet(tan_field, Facet::from(path));
                        }
                    }
                    _ => Err(TantivyError::InvalidArgument(format!(
                        "Invalid data type '{}' for field '{}'",
                        field_type, field_name
                    )))?,
                }
            }
        }

        Ok(compact_doc)
    }
}

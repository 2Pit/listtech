use crate::api;
use crate::api::FieldValue;

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
use tantivy::schema::document::TantivyDocument;
use tantivy::schema::{Facet, Schema as TantivySchema};
use tantivy::schema::{Field, FieldType as TantivyFieldType};

use super::delta_schema::DeltaSchema;

use tantivy::schema::Field as Idx;

#[derive(Debug, Clone)]
pub struct MetaSchema {
    pub name: String,
    pub id_column: MetaColumn,
    idx_by_name: HashMap<String, Idx>,
    columns: Vec<MetaColumn>,
}

#[derive(Debug, Clone)]
pub struct MetaColumn {
    pub idx: Idx, // Tantivy Field
    pub name: String,
    pub tantivy_type: TantivyFieldType, // Tantivy FieldType
    pub column_type: api::ColumnType,   // Наш тип
    pub is_id: bool,
    pub is_nullable: bool,
    pub is_eq: bool,
    pub is_sort_ragne: bool,
}

impl MetaSchema {
    pub fn get_idx(&self, name: &str) -> Result<Idx> {
        self.idx_by_name
            .get(name)
            .map(|idx| idx.clone())
            .ok_or(anyhow!("Unknown column name: {}", name))
    }

    pub fn get_column_type(&self, name: &str) -> Result<api::ColumnType> {
        self.get_idx(name)
            .map(|idx| self.columns[idx.field_id() as usize].column_type.clone())
    }

    pub fn from_tantivy_and_delta(
        tantivy_schema: &TantivySchema,
        delta: DeltaSchema,
    ) -> Result<Self> {
        let mut columns = Vec::new();
        let mut idx_by_name = HashMap::new();
        let mut id_column = None;

        for delta_col in &delta.columns {
            let idx = tantivy_schema
                .get_field(&delta_col.name)
                .map_err(|_| anyhow!("Field '{}' not found in Tantivy schema", delta_col.name))?;

            let field_entry = tantivy_schema.get_field_entry(idx);
            let tantivy_type = field_entry.field_type().clone();

            let is_eq = matches!(
                tantivy_type,
                TantivyFieldType::Str(_) | TantivyFieldType::Facet(_)
            );
            let is_sort_ragne = matches!(
                tantivy_type,
                TantivyFieldType::U64(_)
                    | TantivyFieldType::I64(_)
                    | TantivyFieldType::F64(_)
                    | TantivyFieldType::Date(_)
            );

            let meta_column = MetaColumn {
                idx,
                name: delta_col.name.clone(),
                tantivy_type,
                column_type: delta_col.column_type.clone(),
                is_id: delta_col.is_id,
                is_nullable: delta_col.is_nullable,
                is_eq,
                is_sort_ragne,
            };

            if delta_col.is_id {
                if id_column.is_some() {
                    return Err(anyhow!("Multiple ID columns defined"));
                }
                id_column = Some(meta_column.clone());
            }

            idx_by_name.insert(delta_col.name.clone(), idx);
            columns.push(meta_column);
        }

        let id_column = id_column.ok_or_else(|| anyhow!("No ID column defined"))?;

        Ok(Self {
            name: delta.name.clone(),
            id_column,
            columns,
            idx_by_name,
        })
    }

    pub fn to_tantivy_doc(&self, doc: &api::Document) -> Result<TantivyDocument> {
        let mut compact_doc = TantivyDocument::new();

        for field in &doc.fields {
            let field_name = &field.name;
            let idx = self.get_idx(field_name)?;

            if let Some(ref value) = field.value {
                use api::FieldValue::*;

                let field_type = self.get_column_type(field_name)?;

                match (value, field_type) {
                    (Bool(b), api::ColumnType::Bool) => compact_doc.add_bool(idx, *b),
                    (Long(i), api::ColumnType::Long) => compact_doc.add_i64(idx, *i),
                    (Ulong(u), api::ColumnType::Ulong) => compact_doc.add_u64(idx, *u),
                    (Double(f), api::ColumnType::Double) => compact_doc.add_f64(idx, *f),
                    (String(s), api::ColumnType::String) => compact_doc.add_text(idx, s),
                    (Bytes(b), api::ColumnType::Bytes) => compact_doc.add_bytes(idx, b.as_slice()),
                    (DateTime(iso_date), api::ColumnType::DateTime) => {
                        let dt: chrono::DateTime<chrono::Utc> = iso_date.parse().map_err(|_| {
                            TantivyError::InvalidArgument(format!(
                                "Invalid ISO8601 date: {}",
                                iso_date
                            ))
                        })?;
                        compact_doc.add_date(
                            idx,
                            tantivy::DateTime::from_timestamp_micros(dt.timestamp_micros()),
                        )
                    }
                    (Tree(paths), api::ColumnType::Tree) => {
                        for path in paths {
                            compact_doc.add_facet(idx, tantivy::schema::Facet::from(path));
                        }
                    }
                    _ => {
                        return Err(TantivyError::InvalidArgument(format!(
                            "Invalid data type '{:?}' for field '{}'",
                            value, field_name
                        ))
                        .into());
                    }
                }
            }
        }

        Ok(compact_doc)
    }
}

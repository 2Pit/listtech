use anyhow::{Result, anyhow};
use corelib::model::meta_schema::MetaSchema;
use corelib::trace_err;
use std::collections::HashSet;
use tantivy::TantivyError;
use tantivy::schema::document::TantivyDocument;

use crate::api;
use crate::api::FieldValue::*;

#[tracing::instrument(level = "debug")]
pub fn to_tantivy_doc(meta_schema: &MetaSchema, doc: &api::Document) -> Result<TantivyDocument> {
    let mut compact_doc = TantivyDocument::new();
    let mut indexed_fields = HashSet::new();

    for (field_name, value) in doc
        .fields
        .iter()
        .filter_map(|field| field.value.as_ref().map(|v| (&field.name, v)))
    {
        indexed_fields.insert(field_name.clone());

        let meta_col = meta_schema.get_column(field_name)?;
        let idx = meta_col.idx;
        let column_type = meta_col.column_type;

        match (value, column_type) {
            (Bool(b), api::MetaColumnType::Bool) => compact_doc.add_bool(idx, *b),
            (Long(i), api::MetaColumnType::Long) => compact_doc.add_i64(idx, *i),
            (Ulong(u), api::MetaColumnType::Ulong) => compact_doc.add_u64(idx, *u),
            (Double(f), api::MetaColumnType::Double) => compact_doc.add_f64(idx, *f),
            (Text(s), api::MetaColumnType::Text) => compact_doc.add_text(idx, s),
            (Bytes(b), api::MetaColumnType::Bytes) => compact_doc.add_bytes(idx, b.as_slice()),
            (DateTime(iso_date), api::MetaColumnType::DateTime) => {
                let dt: chrono::DateTime<chrono::Utc> = iso_date.parse().map_err(|_| {
                    TantivyError::InvalidArgument(format!("Invalid ISO8601 date: {}", iso_date))
                })?;
                compact_doc.add_date(
                    idx,
                    tantivy::DateTime::from_timestamp_micros(dt.timestamp_micros()),
                )
            }
            (Tree(paths), api::MetaColumnType::Tree) => {
                for path in paths {
                    compact_doc.add_facet(idx, tantivy::schema::Facet::from(path));
                }
            }
            _ => {
                return trace_err!(Err(TantivyError::InvalidArgument(format!(
                    "Invalid data {}: {} != {:?}",
                    field_name, column_type, value
                ))
                .into()));
            }
        }
    }

    // Ensure all not-null fields are present in the document
    for col in &meta_schema.columns {
        if col.is_not_nullable() && !indexed_fields.contains(&col.name) {
            return Err(anyhow!("Missing required not-null field: '{}'", col.name));
        }
    }

    Ok(compact_doc)
}

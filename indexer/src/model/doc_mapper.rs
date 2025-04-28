use crate::api;
use anyhow::Result;
use corelib::model::meta_schema::MetaSchema;
use tantivy::TantivyError;
use tantivy::schema::document::TantivyDocument;

pub fn to_tantivy_doc(meta_schema: &MetaSchema, doc: &api::Document) -> Result<TantivyDocument> {
    let mut compact_doc = TantivyDocument::new();

    for field in &doc.fields {
        let field_name = &field.name;
        let idx = meta_schema.get_idx(field_name)?;

        if let Some(ref value) = field.value {
            use api::FieldValue::*;

            let colunm_type = meta_schema.get_column_type(field_name)?;

            match (value, colunm_type) {
                (Bool(b), api::MetaColumnType::Bool) => compact_doc.add_bool(idx, *b),
                (Long(i), api::MetaColumnType::Long) => compact_doc.add_i64(idx, *i),
                (Ulong(u), api::MetaColumnType::Ulong) => compact_doc.add_u64(idx, *u),
                (Double(f), api::MetaColumnType::Double) => compact_doc.add_f64(idx, *f),
                (String(s), api::MetaColumnType::String) => compact_doc.add_text(idx, s),
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

use corelib::api::indexer_model::document::*;

use chrono::{DateTime as CronoDateTime, Utc};
use tantivy::DateTime as TantivyDateTime;
use tantivy::TantivyError;
use tantivy::schema::document::TantivyDocument;
use tantivy::schema::{Facet, Schema, Type};

pub fn map_proto_to_tantivy_doc(
    doc: &Document,
    schema: &Schema,
) -> Result<TantivyDocument, TantivyError> {
    let mut compact_doc = TantivyDocument::new();

    for field in &doc.fields {
        let field_name = &field.name;
        let field_entry = schema.get_field(field_name)?;

        if let Some(ref v) = field.value {
            use FieldValue::*;

            let field_type = schema
                .get_field(field_name)
                .map(|name| schema.get_field_entry(name))
                .map(|enty| enty.field_type())
                .map(|ft| ft.value_type())?;

            match v {
                Bool(b) if field_type == Type::Bool => compact_doc.add_bool(field_entry, *b),
                Long(i) if field_type == Type::I64 => compact_doc.add_i64(field_entry, *i),
                Ulong(u) if field_type == Type::U64 => compact_doc.add_u64(field_entry, *u),
                Double(f) if field_type == Type::F64 => compact_doc.add_f64(field_entry, *f),
                String(s) if field_type == Type::Str => compact_doc.add_text(field_entry, s),
                Bytes(b) if field_type == Type::Bytes => {
                    compact_doc.add_bytes(field_entry, b.as_slice())
                }
                DateTime(iso_date) if field_type == Type::Date => {
                    let dt: CronoDateTime<Utc> = iso_date.parse().expect("Invalid ISO 8601 format");
                    compact_doc.add_date(
                        field_entry,
                        TantivyDateTime::from_timestamp_micros(dt.timestamp_micros()),
                    )
                }
                Tree(facest) if field_type == Type::Facet => {
                    for facet_str in facest.into_iter() {
                        compact_doc.add_facet(field_entry, Facet::from(facet_str));
                    }
                }
                _ => Err(TantivyError::InvalidArgument(format!(
                    "Invalid data type '{}' for field '{}'",
                    field_type.to_code(),
                    field_name
                )))?,
            }
        }
    }

    Ok(compact_doc)
}

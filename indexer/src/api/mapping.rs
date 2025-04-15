use crate::api::proto::indexer::indexable_field::Value;
use crate::api::proto::indexer::Document;
use tantivy::schema::document::TantivyDocument;
use tantivy::schema::{Facet, Schema, Type};
use tantivy::DateTime;
use tantivy::TantivyError;

pub fn map_proto_to_tantivy_doc(
    doc: &Document,
    schema: &Schema,
) -> Result<TantivyDocument, TantivyError> {
    let mut compact_doc = TantivyDocument::new();

    for field in &doc.fields {
        let field_name = &field.name;
        let field_entry = schema.get_field(field_name)?;

        if let Some(ref v) = field.value {
            use Value::*;

            let field_type = schema
                .get_field(field_name)
                .map(|name| schema.get_field_entry(name))
                .map(|enty| enty.field_type())
                .map(|ft| ft.value_type())?;

            match v {
                BoolValue(b) if field_type == Type::Bool => compact_doc.add_bool(field_entry, *b),
                LongValue(i) if field_type == Type::I64 => compact_doc.add_i64(field_entry, *i),
                UlongValue(u) if field_type == Type::U64 => compact_doc.add_u64(field_entry, *u),
                DoubleValue(f) if field_type == Type::F64 => compact_doc.add_f64(field_entry, *f),
                StringValue(s) if field_type == Type::Str => compact_doc.add_text(field_entry, s),
                BytesValue(b) if field_type == Type::Bytes => {
                    compact_doc.add_bytes(field_entry, b.as_slice())
                }
                TimestampMsValue(t) if field_type == Type::Date => {
                    compact_doc.add_date(field_entry, DateTime::from_timestamp_nanos(*t))
                }
                _ => Err(TantivyError::InvalidArgument(format!(
                    "Invalid data type '{}' for field '{}'",
                    field_type.to_code(),
                    field_name
                )))?,
            }
        }

        for facet_str in &field.facets {
            compact_doc.add_facet(field_entry, Facet::from(facet_str));
        }
    }

    Ok(compact_doc)
}

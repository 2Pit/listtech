use crate::api::proto::indexer::indexable_field::Value;
use crate::api::proto::indexer::Document;
use tantivy::schema::document::TantivyDocument;
use tantivy::schema::{Facet, Schema};
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

            match v {
                BoolValue(b) => compact_doc.add_bool(field_entry, *b),
                LongValue(i) => compact_doc.add_i64(field_entry, *i),
                UlongValue(u) => compact_doc.add_u64(field_entry, *u),
                DoubleValue(f) => compact_doc.add_f64(field_entry, *f),
                StringValue(s) => compact_doc.add_text(field_entry, s),
                BytesValue(b) => compact_doc.add_bytes(field_entry, b),
                TimestampMsValue(t) => {
                    compact_doc.add_date(field_entry, DateTime::from_timestamp_nanos(*t))
                }
            }
        }

        for facet_str in &field.facets {
            compact_doc.add_facet(field_entry, Facet::from(facet_str));
        }
    }

    Ok(compact_doc)
}

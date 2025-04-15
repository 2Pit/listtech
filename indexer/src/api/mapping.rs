use crate::api::proto::indexer::indexable_field::Value;
use crate::api::proto::indexer::Document;
use tantivy::schema::document::TantivyDocument;
use tantivy::schema::{Facet, Schema};
use tantivy::TantivyError;

pub fn map_proto_to_tantivy(
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
                StringValue(s) => {
                    if field.is_facet {
                        compact_doc.add_facet(field_entry, Facet::from(s));
                    } else {
                        compact_doc.add_text(field_entry, s);
                    }
                }
                IntValue(i) => {
                    compact_doc.add_i64(field_entry, *i);
                }
                DoubleValue(f) => {
                    compact_doc.add_f64(field_entry, *f);
                }
                BoolValue(b) => {
                    compact_doc.add_bool(field_entry, *b);
                }
            }
        }

        // Обработка facets и repeated
        for facet_str in &field.facets {
            compact_doc.add_facet(field_entry, Facet::from(facet_str));
        }
    }

    Ok(compact_doc)
}

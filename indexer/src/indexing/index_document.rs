use super::index_state::IndexState;
use crate::api::mapping::map_proto_to_tantivy_doc;
use crate::api::proto::indexer::indexable_field::Value;
use crate::api::proto::indexer::Document;
use anyhow::{Context, Result};
use tantivy::schema::Term;

pub async fn index_document(index: &IndexState, doc: Document) -> Result<()> {
    let asin_opt = doc
        .fields
        .iter()
        .find(|f| f.name == "asin")
        .and_then(|f| f.value.as_ref())
        .and_then(|v| match v {
            Value::StringValue(s) => Some(s.clone()),
            _ => None,
        });

    let tantivy_doc =
        map_proto_to_tantivy_doc(&doc, &index.schema).context("invalid document structure")?;

    let writer = index.writer.lock().await;

    if let Some(asin) = asin_opt {
        let asin_field = index
            .schema
            .get_field("asin")
            .context("asin field not found in schema")?;

        let term = Term::from_field_text(asin_field, &asin);
        writer.delete_term(term);
    }

    writer
        .add_document(tantivy_doc)
        .context("failed to add document")?;

    Ok(())
}

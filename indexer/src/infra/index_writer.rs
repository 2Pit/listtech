use crate::api::mapping::map_proto_to_tantivy_doc;
use crate::infra::index::IndexState;
use corelib::api::indexer_model::document::*;

use anyhow::{Context, Result};
use tantivy::schema::Term;

pub async fn add_document_safely(index_state: &IndexState, doc: Document) -> Result<()> {
    let tantivy_doc = map_proto_to_tantivy_doc(&doc, &index_state.index.schema())
        .context("invalid document structure")?;

    let writer = index_state.writer.lock().await;

    let id_col = &index_state.schema.id_column;
    let term = doc
        .fields
        .into_iter()
        .find(|field| field.name == id_col.name)
        .ok_or_else(|| anyhow::anyhow!("ID not found"))
        .and_then(|field| field.value.ok_or_else(|| anyhow::anyhow!("ID is null")))
        .and_then(|id_value| match id_value {
            FieldValue::String(id) => Ok(Term::from_field_text(id_col.tan_field, id.as_str())),
            FieldValue::Long(id) => Ok(Term::from_field_i64(id_col.tan_field, id)),
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

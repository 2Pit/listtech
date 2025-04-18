use crate::api::mapping::map_proto_to_tantivy_doc;
use crate::domain::document::extract_asin;
use crate::infra::index::IndexState;
use corelib::proto::indexer::Document;

use anyhow::{Context, Result};
use tantivy::schema::Term;

/// Индексирует один документ с возможной заменой по asin
pub async fn index_document(index: &IndexState, doc: Document) -> Result<()> {
    let tantivy_doc =
        map_proto_to_tantivy_doc(&doc, &index.schema).context("invalid document structure")?;

    let writer = index.writer.lock().await;

    if let Some(asin) = extract_asin(&doc) {
        let asin_field = index
            .schema
            .get_field("asin")
            .context("asin field not found in schema")?;

        let term = Term::from_field_text(asin_field, asin.as_str());
        writer.delete_term(term);
    }

    writer
        .add_document(tantivy_doc)
        .context("failed to add document")?;

    Ok(())
}

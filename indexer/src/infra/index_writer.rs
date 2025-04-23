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

pub async fn add_document_safely(index: &IndexState, doc: Document) -> anyhow::Result<()> {
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

    loop {
        match writer.add_document(tantivy_doc.clone()) {
            Ok(_) => break,
            Err(e) => return Err(e.into()),
        }
    }

    // let count = index.doc_counter.fetch_add(1, Ordering::Relaxed) + 1;
    // if count % 1000 == 0 {
    //     writer.commit()?;
    //     tracing::info!(doc_count = count, "Committed after 1000 documents");
    // }

    Ok(())
}

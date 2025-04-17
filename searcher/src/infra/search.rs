use crate::domain::document::to_string_owned_value;
use crate::infra::index::SearchIndex;
use std::collections::HashMap;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, OwnedValue},
};
use tonic::Status;

use crate::api::proto::searcher::SearchHit;

/// Выполняет простой BM25-поиск по полям title и description.
pub fn execute_search(index: &SearchIndex, query_str: &str) -> Result<Vec<SearchHit>, Status> {
    let searcher = index.reader.searcher();
    let schema = index.index.schema();

    let default_fields: Vec<Field> = ["title", "description"]
        .iter()
        .filter_map(|&name| schema.get_field(name).ok())
        .collect();

    let parser = QueryParser::for_index(&index.index, default_fields);
    let query = parser
        .parse_query(query_str)
        .map_err(|e| Status::invalid_argument(format!("Invalid query: {e}")))?;

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(10))
        .map_err(|e| Status::internal(format!("Search failed: {e}")))?;

    let mut hits = Vec::new();

    for (_score, addr) in top_docs {
        let retrieved: HashMap<Field, OwnedValue> = searcher
            .doc(addr)
            .map_err(|e| Status::internal(format!("Failed to retrieve document: {e}")))?;

        let mut fields = HashMap::new();

        for (field, value) in retrieved.iter() {
            let field_name = schema.get_field_name(*field);
            fields.insert(field_name.to_string(), to_string_owned_value(value));
        }

        hits.push(SearchHit {
            doc_id: "some_doc_id".to_string(),
            fields,
        });
    }

    Ok(hits)
}

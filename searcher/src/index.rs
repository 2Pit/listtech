use anyhow::{Context, Result};
use std::path::Path;
use tantivy::{Index, IndexReader, ReloadPolicy};
use tonic::{Request, Response, Status};

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Field;
use tantivy::schema::OwnedValue;

use std::collections::HashMap;

use crate::api::proto::searcher::{
    search_service_server::SearchService, SearchHit, SearchRequest, SearchResponse,
};

pub struct SearchIndex {
    pub index: Index,
    pub reader: IndexReader,
}

impl SearchIndex {
    pub fn open_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let index = Index::open_in_dir(&path)
            .with_context(|| format!("Failed to open index in {:?}", path.as_ref()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create IndexReader")?;

        Ok(Self { index, reader })
    }
}

#[tonic::async_trait]
impl SearchService for SearchIndex {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let query_str = request.into_inner().query;

        let reader = &self.reader;
        let searcher = reader.searcher();
        let schema = self.index.schema();

        // поля, по которым ищем
        let default_fields: Vec<Field> = ["title", "description"]
            .iter()
            .filter_map(|&name| schema.get_field(name).ok())
            .collect();

        let parser = QueryParser::for_index(&self.index, default_fields);
        let query = parser
            .parse_query(&query_str)
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
                doc_id: "doc_id".to_string(),
                fields,
            });
        }

        Ok(Response::new(SearchResponse { hits }))
    }
}

pub fn to_string_owned_value(value: &OwnedValue) -> String {
    match value {
        OwnedValue::Null => "null".to_string(),
        OwnedValue::Str(s) => format!("\"{}\"", s),
        OwnedValue::PreTokStr(p) => format!("\"{}\"", p.text),
        OwnedValue::U64(n) => n.to_string(),
        OwnedValue::I64(n) => n.to_string(),
        OwnedValue::F64(n) => n.to_string(),
        OwnedValue::Bool(b) => b.to_string(),
        OwnedValue::Date(dt) => dt.into_timestamp_millis().to_string(),
        OwnedValue::Facet(f) => format!("\"{}\"", f.to_path_string()),
        OwnedValue::Bytes(_) => "hex".to_string(), //format!("0x{}", hex::encode(bytes)),
        OwnedValue::Array(arr) => {
            let inner = arr
                .iter()
                .map(to_string_owned_value)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", inner)
        }
        OwnedValue::Object(obj) => {
            let inner = obj
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, to_string_owned_value(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", inner)
        }
        OwnedValue::IpAddr(addr) => format!("\"{}\"", addr),
    }
}

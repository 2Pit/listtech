use anyhow::{Result, anyhow};
use tantivy::query::QueryParser;
use tantivy::{DocAddress, Score, collector::TopDocs};
use tonic::Status;
use tracing::info;

use crate::api;
use crate::domain::index::SearchIndex;

use super::virtual_sort::collector::SortByVirtualFieldCollector;
use super::virtual_sort::expr::Expr;
use super::virtual_sort::program::Program;

pub fn execute_search(
    index: &SearchIndex,
    req: &api::SearchRequest,
) -> Result<Vec<(Score, DocAddress)>, Status> {
    let searcher = index.reader.searcher();
    // let schema = index.index.schema();

    // let default_fields = index.schema.columns.iter().map(|c| c.idx).collect();
    let default_fields = index.schema.get_full_text_col_idx();
    let parser = QueryParser::for_index(&index.index, default_fields);
    let query = parser
        .parse_query(&req.filter)
        .map_err(|e| Status::invalid_argument(format!("Invalid query: {e}")))?;

    let top_docs = match &req.sort {
        Some(sort_func) => {
            info!("USED sort_func");
            let program = parse_and_compile_program(&sort_func)
                .map_err(|e| Status::internal(format!("Search failed: {e}")))?;

            let collector = SortByVirtualFieldCollector {
                limit: req.limit,
                offset: req.offset,
                program,
                schema: &index.schema,
            };

            searcher
                .search(&query, &collector)
                .map_err(|e| Status::internal(format!("Search failed: {e}")))?
        }
        None => {
            info!("TOP_N sort");
            let collector = TopDocs::with_limit(req.limit).and_offset(req.offset);

            searcher
                .search(&query, &collector)
                .map_err(|e| Status::internal(format!("Search failed: {e}")))?
        }
    };

    Ok(top_docs)
}

fn parse_and_compile_program(func: &str) -> Result<Program> {
    let expr = Expr::parse(func).into_result().map_err(|errs| {
        anyhow!(
            "Failed to parse function: {}",
            errs.into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    Ok(Program::compile_expr(expr))
}

use crate::api::{Column, SearchField};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub select: String,
    pub filter: String,
    pub projections: Vec<String>,
    // pub group_by: Vec<String>,
    // pub sort_by: Vec<(String, bool)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub fields: Vec<SearchField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatrixResponse {
    pub row_count: u32,
    pub columns: Vec<Column>,
}

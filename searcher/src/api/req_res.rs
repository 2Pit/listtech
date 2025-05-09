use crate::api::{Column, SearchField};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub select: Vec<String>,
    pub from: String,
    pub filter: String,

    #[serde(default)]
    pub sort: Option<String>,

    #[serde(default)]
    pub functions: Vec<String>,

    #[serde(default)]
    pub offset: usize,

    #[serde(default = "default_limit")]
    pub limit: usize,
}

const fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    pub fields: Vec<SearchField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatrixResponse {
    pub row_count: u32,
    pub columns: Vec<Column>,
}

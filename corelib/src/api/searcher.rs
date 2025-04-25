use serde::{Deserialize, Serialize};

pub struct SearchQuery {
    pub filter: String,
    pub projections: Vec<String>,
    pub group_by: Vec<String>,
    pub sort_by: Vec<(String, bool)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub fields: Vec<SearchField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchField {
    pub name: String,
    pub value: SearchValue,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SearchValue {
    Bool(bool),
    Ulong(u64),
    Long(i64),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
    DateTime(String),
    Facet(Vec<String>),

    NullableBool(Option<bool>),
    NullableUlong(Option<u64>),
    NullableLong(Option<i64>),
    NullableDouble(Option<f64>),
    NullableString(Option<String>),
    NullableBytes(Option<Vec<u8>>),
    NullableDateTime(Option<String>),
    NullableFacet(Option<Vec<String>>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatrixResponse {
    pub row_count: u32,
    pub columns: Vec<Column>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub values: ColumnValues,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "values")]
pub enum ColumnValues {
    Bool(Vec<bool>),
    UInt64(Vec<u64>),
    Int64(Vec<i64>),
    Double(Vec<f64>),
    String(Vec<String>),
    Bytes(Vec<Vec<u8>>),
    DateTime(Vec<String>),
    Facet(Vec<String>),

    NullableBool(Vec<Option<bool>>),
    NullableUInt64(Vec<Option<u64>>),
    NullableInt64(Vec<Option<i64>>),
    NullableDouble(Vec<Option<f64>>),
    NullableString(Vec<Option<String>>),
    NullableBytes(Vec<Option<Vec<u8>>>),
    NullableDateTime(Vec<Option<String>>),
    NullableFacet(Vec<Option<String>>),
}

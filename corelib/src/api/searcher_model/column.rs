use serde::{Deserialize, Serialize};

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

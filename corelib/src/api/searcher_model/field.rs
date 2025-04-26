use serde::{Deserialize, Serialize};

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

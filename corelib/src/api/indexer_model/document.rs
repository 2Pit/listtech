use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub index_name: String,
    pub index_version: u32,
    pub fields: Vec<IndexableField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexableField {
    pub name: String,
    pub value: Option<FieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum FieldValue {
    // value types
    Bool(bool),
    Ulong(u64),
    Long(i64),
    Double(f64),
    DateTime(String),
    // object types
    Bytes(Vec<u8>),
    Facet(Vec<String>),
    String(String),
}

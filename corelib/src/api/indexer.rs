use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    pub document: Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentResponse;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddDocumentsRequest {
//     pub documents: Vec<Document>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddDocumentsResponse;

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
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    // value types (zero_indexed)
    Bool,
    Ulong,
    Long,
    Double,
    DateTime, // 0 => 0 epoch

    // object types
    Bytes,  // 0 => []
    Facet,  // 0 => []
    String, // 0 => ""
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FieldModifier {
    // about value
    ZeroIndexed,
    ZeroDefault,

    // about null as value
    Nullable,
    NullIndexed,
    NullDefault,
}

// ------- Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaRequest {
    pub document: Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDescriptor {
    pub name: String,
    pub vesion: u32,
    pub fields: Vec<FieldDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDescriptor {
    pub name: String,
    pub field_type: FieldType,
    pub modifiers: Vec<FieldModifier>,
}

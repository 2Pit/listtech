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
    Tree(Vec<String>),
    String(String),
}

impl std::fmt::Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Bool(v) => write!(f, "{}", v),
            FieldValue::Ulong(v) => write!(f, "{}", v),
            FieldValue::Long(v) => write!(f, "{}", v),
            FieldValue::Double(v) => write!(f, "{}", v),
            FieldValue::DateTime(v) => write!(f, "{}", v),
            FieldValue::Bytes(v) => write!(f, "{:?}", v), // байты выводим через дебаг
            FieldValue::Tree(v) => write!(f, "[{}]", v.join(", ")), // склеиваем вектор строк через запятую
            FieldValue::String(v) => write!(f, "{}", v),
        }
    }
}

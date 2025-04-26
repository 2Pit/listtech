use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub version: u32,
    pub columns: Vec<ColumnType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnType {
    pub filed_type: FieldType,
    pub modifiers: Vec<FieldModifier>,
    pub on_missing: OnMissing,
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
    String, // 0 => ""
    Bytes,  // 0 => []
    Tree,   // 0 => ["/"]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FieldModifier {
    Stored,
    Equals,
    FastSortable,
    FullText,
    Nullable,
    // Groupable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OnMissing {
    Error,
    Zero,
    Null,
}

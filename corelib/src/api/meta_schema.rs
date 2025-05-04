use anyhow::{Context, Result};
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSchema {
    pub name: String,
    pub columns: Vec<MetaColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaColumn {
    // pub idx: Idx,
    pub name: String,
    pub column_type: MetaColumnType,
    pub modifiers: HashSet<MetaColumnModifier>,
}

impl MetaSchema {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Start reading delta_schema");
        let json_str = fs::read_to_string(path)
            .with_context(|| format!("Failed to read delta schema file: {:?}", path))?;
        info!("Finish reading delta_schema");

        let schema = serde_json::from_str::<MetaSchema>(&json_str)
            .with_context(|| format!("Failed to parse delta schema JSON from file: {:?}", path))?;

        Ok(schema)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum MetaColumnType {
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
pub enum MetaColumnModifier {
    Id,
    // Stored,
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

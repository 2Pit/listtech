use crate::api;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSchema {
    pub name: String,
    pub version: u32,
    pub columns: Vec<DeltaColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaColumn {
    pub name: String,
    pub column_type: api::ColumnType,
    pub is_id: bool,
    pub is_nullable: bool,
}

impl DeltaSchema {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let json_str = fs::read_to_string(path)
            .with_context(|| format!("Failed to read delta schema file: {:?}", path))?;

        let schema = serde_json::from_str::<DeltaSchema>(&json_str)
            .with_context(|| format!("Failed to parse delta schema JSON from file: {:?}", path))?;

        Ok(schema)
    }
}

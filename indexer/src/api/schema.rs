use crate::api::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub version: u32,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub filed_type: MetaColumnType,
    pub modifiers: Vec<MetaColumnModifier>,
    pub on_missing: OnMissing,
}

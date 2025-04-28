use anyhow::Result;
use anyhow::anyhow;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tantivy::schema::IndexRecordOption;
use tantivy::schema::{FieldType as TantivyFieldType, Schema as TantivySchema};

use super::delta_schema::DeltaSchema;

use tantivy::schema::Field as Idx;

#[derive(Debug, Clone)]
pub struct MetaSchema {
    pub name: String,
    pub id_column: MetaColumn,
    idx_by_name: HashMap<String, Idx>,
    columns: Vec<MetaColumn>,
}

#[derive(Debug, Clone)]
pub struct MetaColumn {
    pub idx: Idx,
    pub name: String,
    pub tantivy_type: TantivyFieldType,
    pub column_type: MetaColumnType,
    pub is_id: bool,
    pub is_nullable: bool,
    pub is_eq: bool,
    pub is_sort_range: bool,
    pub is_full_text: bool,
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
    ID,
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

impl MetaSchema {
    pub fn get_idx(&self, name: &str) -> Result<Idx> {
        self.idx_by_name
            .get(name)
            .map(|idx| idx.clone())
            .ok_or(anyhow!("Unknown column name: {}", name))
    }

    pub fn get_column_type(&self, name: &str) -> Result<MetaColumnType> {
        self.get_idx(name)
            .map(|idx| self.columns[idx.field_id() as usize].column_type.clone())
    }

    // todo: cache
    pub fn get_full_text_col_idx(&self) -> Vec<Idx> {
        self.columns
            .iter()
            .filter(|col| col.is_full_text)
            .map(|mc| mc.idx)
            .collect()
    }

    pub fn from_tantivy_and_delta(
        tantivy_schema: &TantivySchema,
        delta: DeltaSchema,
    ) -> Result<Self> {
        let mut columns = Vec::new();
        let mut idx_by_name = HashMap::new();
        let mut id_column = None;

        for delta_col in &delta.columns {
            let idx = tantivy_schema
                .get_field(&delta_col.name)
                .map_err(|_| anyhow!("Field '{}' not found in Tantivy schema", delta_col.name))?;

            let field_entry = tantivy_schema.get_field_entry(idx);
            let (is_eq, is_sort_range) = match field_entry.field_type() {
                TantivyFieldType::Str(opt) => (opt.get_indexing_options().is_some(), opt.is_fast()),
                TantivyFieldType::U64(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::I64(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::F64(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::Bool(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::Date(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::Facet(_) => (true, true),
                TantivyFieldType::Bytes(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::JsonObject(opt) => (opt.is_indexed(), opt.is_fast()),
                TantivyFieldType::IpAddr(opt) => (opt.is_indexed(), opt.is_fast()),
            };

            let is_full_text = match field_entry.field_type() {
                TantivyFieldType::Str(opt) => opt
                    .get_indexing_options()
                    .filter(|i| i.index_option() == IndexRecordOption::WithFreqsAndPositions)
                    .is_some(),
                _ => false,
            };

            let meta_column = MetaColumn {
                idx,
                name: delta_col.name.clone(),
                tantivy_type: field_entry.field_type().clone(),
                column_type: delta_col.column_type.clone(),
                is_id: delta_col.is_id,
                is_nullable: delta_col.is_nullable,
                is_eq,
                is_sort_range,
                is_full_text,
            };

            if delta_col.is_id {
                if id_column.is_some() {
                    return Err(anyhow!("Multiple ID columns defined"));
                }
                id_column = Some(meta_column.clone());
            }

            idx_by_name.insert(delta_col.name.clone(), idx);
            columns.push(meta_column);
        }

        let id_column = id_column.ok_or_else(|| anyhow!("No ID column defined"))?;

        Ok(Self {
            name: delta.name.clone(),
            id_column,
            columns,
            idx_by_name,
        })
    }
}

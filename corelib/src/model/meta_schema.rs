use anyhow::Result;
use anyhow::anyhow;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use tantivy::schema::IndexRecordOption;
use tantivy::schema::{FieldType as TantivyFieldType, Schema as TantivySchema};

use super::delta_schema::DeltaColumn;
use super::delta_schema::DeltaSchema;

use tantivy::schema::Field as Idx;

#[derive(Debug, Clone)]
pub struct MetaSchema {
    pub name: String,
    pub id_column: MetaColumn,
    pub idx_by_name: HashMap<String, Idx>,
    pub columns: Vec<MetaColumn>,
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

    pub fn get_column(&self, name: &str) -> Result<&MetaColumn> {
        self.get_idx(name)
            .map(|idx| &self.columns[idx.field_id() as usize])
    }

    pub fn from_tantivy_and_delta(
        tantivy_schema: &TantivySchema,
        delta: DeltaSchema,
    ) -> Result<Self> {
        let mut columns = Vec::new();
        let mut idx_by_name = HashMap::new();
        let mut id_column = None;

        let col_map = delta
            .columns
            .iter()
            .map(|dc| (dc.name.as_str(), dc))
            .collect::<HashMap<_, _>>();

        for (idx, field_entry) in tantivy_schema.fields() {
            let field_name = field_entry.name();

            let delta_col_opt = col_map.get(field_name);

            let (is_eq, is_sort_range, meta_col_type) = match field_entry.field_type() {
                TantivyFieldType::Str(opt) => (
                    opt.get_indexing_options().is_some(),
                    opt.is_fast(),
                    MetaColumnType::Bool,
                ),
                TantivyFieldType::U64(opt) => {
                    (opt.is_indexed(), opt.is_fast(), MetaColumnType::Ulong)
                }
                TantivyFieldType::I64(opt) => {
                    (opt.is_indexed(), opt.is_fast(), MetaColumnType::Long)
                }
                TantivyFieldType::F64(opt) => {
                    (opt.is_indexed(), opt.is_fast(), MetaColumnType::Double)
                }
                TantivyFieldType::Bool(opt) => {
                    (opt.is_indexed(), opt.is_fast(), MetaColumnType::Bool)
                }
                TantivyFieldType::Date(opt) => {
                    (opt.is_indexed(), opt.is_fast(), MetaColumnType::DateTime)
                }
                TantivyFieldType::Facet(_) => (true, true, MetaColumnType::Tree),
                TantivyFieldType::Bytes(opt) => {
                    (opt.is_indexed(), opt.is_fast(), MetaColumnType::Bytes)
                }
                _ => {
                    return Err(anyhow!(
                        "Unsupported field type" // field_entry.field_type()
                    ));
                } // TantivyFieldType::JsonObject(opt) => (opt.is_indexed(), opt.is_fast()),
                  // TantivyFieldType::IpAddr(opt) => (opt.is_indexed(), opt.is_fast()),
            };

            let is_full_text = match field_entry.field_type() {
                TantivyFieldType::Str(opt) => opt
                    .get_indexing_options()
                    .filter(|i| i.index_option() == IndexRecordOption::WithFreqsAndPositions)
                    .is_some(),
                _ => false,
            };

            // Если в дельте нет — ставим дефолтные значения
            let delta_col = if let Some(delta_col) = delta_col_opt {
                (**delta_col).clone()
            } else {
                DeltaColumn {
                    name: field_name.to_string(),
                    column_type: meta_col_type,
                    is_id: false,
                    is_nullable: false,
                }
            };

            let meta_column = MetaColumn {
                idx,
                name: field_name.to_string(),
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

            idx_by_name.insert(field_name.to_string(), idx);
            columns.push(meta_column);
        }

        let id_column = id_column.ok_or_else(|| anyhow!("No ID column defined"))?;

        Ok(Self {
            name: delta.name,
            id_column,
            columns,
            idx_by_name,
        })
    }
}

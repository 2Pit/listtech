use anyhow::Result;
use anyhow::anyhow;
use std::collections::HashMap;
use std::collections::HashSet;
use tantivy::schema::IndexRecordOption;
use tantivy::schema::NumericOptions;
use tantivy::schema::*;
use tantivy::schema::{FieldType as TantivyFieldType, Schema as TantivySchema};

use crate::api;

use tantivy::schema::Field as Idx;

#[derive(Debug, Clone)]
pub struct MetaSchema {
    pub name: String,
    pub columns: Vec<MetaColumn>,

    pub id_column: MetaColumn,
    pub idx_by_name: HashMap<String, Idx>,
}

#[derive(Debug, Clone)]
pub struct MetaColumn {
    pub idx: Idx,
    pub name: String,
    pub column_type: api::MetaColumnType,
    pub is_id: bool,
    pub is_nullable: bool,
    pub is_eq: bool,
    pub is_sort_range: bool,
    pub is_full_text: bool,
    pub tantivy_type: TantivyFieldType,
}

impl MetaSchema {
    pub fn get_idx(&self, name: &str) -> Result<Idx> {
        self.idx_by_name
            .get(name)
            .map(|idx| idx.clone())
            .ok_or(anyhow!("Unknown column name: {}", name))
    }

    pub fn get_column_type(&self, name: &str) -> Result<api::MetaColumnType> {
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

    pub fn build_model_schema(
        tantivy_schema: &TantivySchema,
        api_schema: api::MetaSchema,
    ) -> Result<Self> {
        let mut columns = Vec::new();
        let mut idx_by_name = HashMap::new();
        let mut id_column = None;

        let col_map = api_schema
            .columns
            .iter()
            .map(|dc| (dc.name.as_str(), dc))
            .collect::<HashMap<_, _>>();

        for (idx, field_entry) in tantivy_schema.fields() {
            let field_name = field_entry.name();

            let api_column_opt = col_map.get(field_name);

            let (is_eq, is_sort_range, meta_col_type) = match field_entry.field_type() {
                TantivyFieldType::Str(opt) => (
                    opt.get_indexing_options().is_some(),
                    opt.is_fast(),
                    api::MetaColumnType::Bool,
                ),
                TantivyFieldType::U64(opt) => {
                    (opt.is_indexed(), opt.is_fast(), api::MetaColumnType::Ulong)
                }
                TantivyFieldType::I64(opt) => {
                    (opt.is_indexed(), opt.is_fast(), api::MetaColumnType::Long)
                }
                TantivyFieldType::F64(opt) => {
                    (opt.is_indexed(), opt.is_fast(), api::MetaColumnType::Double)
                }
                TantivyFieldType::Bool(opt) => {
                    (opt.is_indexed(), opt.is_fast(), api::MetaColumnType::Bool)
                }
                TantivyFieldType::Date(opt) => (
                    opt.is_indexed(),
                    opt.is_fast(),
                    api::MetaColumnType::DateTime,
                ),
                TantivyFieldType::Facet(_) => (true, true, api::MetaColumnType::Tree),
                TantivyFieldType::Bytes(opt) => {
                    (opt.is_indexed(), opt.is_fast(), api::MetaColumnType::Bytes)
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
            let api_column = if let Some(api_column) = api_column_opt {
                (**api_column).clone()
            } else {
                api::MetaColumn {
                    // idx,
                    name: field_name.to_string(),
                    column_type: meta_col_type,
                    modifiers: HashSet::new(),
                }
            };

            let meta_column = MetaColumn {
                idx,
                name: field_name.to_string(),
                tantivy_type: field_entry.field_type().clone(),
                column_type: api_column.column_type.clone(),
                is_id: api_column.modifiers.contains(&api::MetaColumnModifier::Id),
                is_nullable: api_column
                    .modifiers
                    .contains(&api::MetaColumnModifier::Nullable),
                is_eq,
                is_sort_range,
                is_full_text,
            };

            if api_column.modifiers.contains(&api::MetaColumnModifier::Id) {
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
            name: api_schema.name,
            id_column,
            columns,
            idx_by_name,
        })
    }
}

// impl From<api::MetaSchema> for MetaSchema {
//     fn from(value: api::MetaSchema) -> Self {
//         let mut id_column = None;
//         let mut idx_by_name = HashMap::new();

//         for col in value.columns.iter() {
//             idx_by_name.insert(col.name.clone(), col.idx);
//             if col.modifiers.contains(&api::MetaColumnModifier::Id) {
//                 id_column = Some(MetaColumn::from(col.clone()));
//             }
//         }

//         let id_column = id_column.unwrap();
//         // .ok_or_else(|| serde::de::Error::custom("Missing column with is_id = true"))?;

//         MetaSchema {
//             name: value.name,
//             columns: value.columns.into_iter().map(MetaColumn::from).collect(),
//             id_column,
//             idx_by_name,
//         }
//     }
// }

// impl From<api::MetaColumn> for MetaColumn {
//     fn from(raw: api::MetaColumn) -> Self {
//         let is_id = raw.modifiers.contains(&api::MetaColumnModifier::Id);
//         let is_eq = raw.modifiers.contains(&api::MetaColumnModifier::Equals);
//         let is_sort_range = raw
//             .modifiers
//             .contains(&api::MetaColumnModifier::FastSortable);
//         let is_full_text = raw.modifiers.contains(&api::MetaColumnModifier::FullText);
//         let is_nullable = raw.modifiers.contains(&api::MetaColumnModifier::Nullable);

//         let tantivy_type = match raw.column_type {
//             api::MetaColumnType::Bool
//             | api::MetaColumnType::Ulong
//             | api::MetaColumnType::Long
//             | api::MetaColumnType::Double => {
//                 let mut opt = NumericOptions::from(STORED);
//                 if is_eq {
//                     opt = opt | NumericOptions::from(INDEXED)
//                 };
//                 if is_sort_range {
//                     opt = opt | NumericOptions::from(FAST)
//                 };
//                 match raw.column_type {
//                     api::MetaColumnType::Bool => TantivyFieldType::Bool(opt),
//                     api::MetaColumnType::Ulong => TantivyFieldType::U64(opt),
//                     api::MetaColumnType::Long => TantivyFieldType::I64(opt),
//                     api::MetaColumnType::Double => TantivyFieldType::F64(opt),
//                     _ => todo!(),
//                 }
//             }

//             api::MetaColumnType::DateTime => {
//                 let mut opt = DateOptions::from(STORED);
//                 if is_eq {
//                     opt = opt | DateOptions::from(INDEXED)
//                 };
//                 if is_sort_range {
//                     opt = opt | DateOptions::from(FAST)
//                 };
//                 TantivyFieldType::Date(opt)
//             }
//             api::MetaColumnType::String => {
//                 let mut opt = TextOptions::from(STORED);
//                 if is_eq {
//                     opt = opt | TextOptions::from(STRING)
//                 };
//                 if is_sort_range {
//                     opt = opt | TextOptions::from(FAST)
//                 };
//                 if is_full_text {
//                     opt = opt | TextOptions::from(TEXT)
//                 };
//                 TantivyFieldType::Str(opt)
//             }
//             api::MetaColumnType::Bytes => {
//                 let mut opt = BytesOptions::from(STORED);
//                 if is_eq {
//                     opt = opt | BytesOptions::from(INDEXED)
//                 };
//                 if is_sort_range {
//                     opt = opt | BytesOptions::from(FAST)
//                 };
//                 TantivyFieldType::Bytes(opt)
//             }
//             api::MetaColumnType::Tree => {
//                 let mut opt = FacetOptions::from(STORED);
//                 if is_eq {
//                     opt = opt | FacetOptions::from(INDEXED)
//                 };
//                 // if let Some(tpe) = col_modif.1 {
//                 //     opt = opt | FacetOptions::from(tpe)
//                 // };
//                 TantivyFieldType::Facet(opt)
//             }
//         };

//         MetaColumn {
//             idx: raw.idx,
//             name: raw.name,
//             column_type: raw.column_type,
//             is_id,
//             is_nullable,
//             is_eq,
//             is_sort_range,
//             is_full_text,
//             tantivy_type,
//         }
//     }
// }

impl Into<api::MetaSchema> for MetaSchema {
    fn into(self) -> api::MetaSchema {
        api::MetaSchema {
            name: self.name,
            columns: self.columns.into_iter().map(MetaColumn::into).collect(),
        }
    }
}

impl Into<api::MetaColumn> for MetaColumn {
    fn into(self) -> api::MetaColumn {
        let modifiers = vec![
            if self.is_id {
                Some(api::MetaColumnModifier::Id)
            } else {
                None
            },
            if self.is_eq {
                Some(api::MetaColumnModifier::Equals)
            } else {
                None
            },
            if self.is_sort_range {
                Some(api::MetaColumnModifier::FastSortable)
            } else {
                None
            },
            if self.is_full_text {
                Some(api::MetaColumnModifier::FullText)
            } else {
                None
            },
            if self.is_nullable {
                Some(api::MetaColumnModifier::Nullable)
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        .collect();

        api::MetaColumn {
            // idx: self.idx,
            name: self.name,
            column_type: self.column_type,
            modifiers,
        }
    }
}

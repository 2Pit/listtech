use anyhow::Result;
use anyhow::anyhow;
use std::collections::HashMap;
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
    pub name: String,
    pub idx: Idx,
    pub tantivy_type: TantivyFieldType,
    pub column_type: api::MetaColumnType,
    pub is_id: bool,
    pub is_nullable: bool,
    pub is_eq: bool,
    pub is_sort_range: bool,
    pub is_full_text: bool,
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

    pub fn from_api(tantivy_schema: &TantivySchema, api_schema: api::MetaSchema) -> Result<Self> {
        let columns: Vec<MetaColumn> = api_schema
            .columns
            .into_iter()
            .map(|api_col| MetaColumn::from_api(api_col, tantivy_schema))
            .collect::<Result<Vec<_>>>()?;

        let id_column = columns
            .iter()
            .find(|col| col.is_id)
            .ok_or(anyhow!("Missing ID column"))?
            .clone();

        let mut idx_by_name = HashMap::new();
        for column in &columns {
            idx_by_name.insert(column.name.clone(), column.idx);
        }

        Ok(Self {
            name: api_schema.name,
            id_column,
            columns,
            idx_by_name,
        })
    }
}

impl MetaColumn {
    fn from_api(api_column: api::MetaColumn, tantivy_schema: &TantivySchema) -> Result<Self> {
        let idx = tantivy_schema.get_field(&api_column.name)?;
        let filed_entry = tantivy_schema.get_field_entry(idx);

        Ok(Self {
            name: api_column.name,
            idx,
            tantivy_type: filed_entry.field_type().clone(),
            column_type: api_column.column_type,
            is_id: api_column.modifiers.contains(&api::MetaColumnModifier::Id),
            is_nullable: api_column
                .modifiers
                .contains(&api::MetaColumnModifier::Nullable),
            is_eq: api_column
                .modifiers
                .contains(&api::MetaColumnModifier::Equals),
            is_sort_range: api_column
                .modifiers
                .contains(&api::MetaColumnModifier::FastSortable),
            is_full_text: api_column
                .modifiers
                .contains(&api::MetaColumnModifier::FullText),
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

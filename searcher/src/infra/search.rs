use anyhow::{Result, anyhow};
use corelib::api::MetaColumnType;
use corelib::model::MetaSchema;
use indexmap::IndexSet;
use std::collections::HashMap;
use tantivy::query::QueryParser;
use tantivy::{
    DocAddress, Score,
    collector::TopDocs,
    schema::{Field, OwnedValue},
};
use tonic::Status;
use tracing::info;

use crate::api::SearchValue::*;
use crate::api::{self, SearchField};
use crate::domain::document::{map_owned_value, owned_val_as_f32};
use crate::infra::index::SearchIndex;

use super::online::evaluation;
use super::online::parsing::Expr;
use super::online::program::{OpCode, Program};
use super::virtual_collector::SortByVirtualFieldCollector;

pub fn execute_search(
    index: &SearchIndex,
    req: &api::SearchRequest,
) -> Result<Vec<(Score, DocAddress)>, Status> {
    let searcher = index.reader.searcher();
    // let schema = index.index.schema();

    // let default_fields = index.schema.columns.iter().map(|c| c.idx).collect();
    let default_fields = index.schema.get_full_text_col_idx();
    let parser = QueryParser::for_index(&index.index, default_fields);
    let query = parser
        .parse_query(&req.filter)
        .map_err(|e| Status::invalid_argument(format!("Invalid query: {e}")))?;

    let top_docs = match &req.sort {
        Some(sort_func) => {
            info!("USED sort_func");
            let program = parse_and_compile_program(&sort_func)
                .map_err(|e| Status::internal(format!("Search failed: {e}")))?;

            let collector = SortByVirtualFieldCollector {
                limit: req.limit,
                offset: req.offset,
                program,
                schema: &index.schema,
            };

            searcher
                .search(&query, &collector)
                .map_err(|e| Status::internal(format!("Search failed: {e}")))?
        }
        None => {
            info!("TOP_N sort");
            let collector = TopDocs::with_limit(req.limit).and_offset(req.offset);

            searcher
                .search(&query, &collector)
                .map_err(|e| Status::internal(format!("Search failed: {e}")))?
        }
    };

    Ok(top_docs)
}

pub fn build_search_response(
    index: &SearchIndex,
    top_docs: &[(Score, DocAddress)],
    req: &api::SearchRequest,
) -> Result<api::SearchResponse> {
    let searcher = index.reader.searcher();
    let schema = &index.schema;

    let mut field_set = IndexSet::new();
    for field in &req.select {
        if field == "*" {
            field_set.extend(index.schema.idx_by_name.keys());
        } else {
            field_set.insert(field);
        }
    }
    let mut rows = Vec::with_capacity(top_docs.len());

    for &(_, addr) in top_docs {
        let mut fields = Vec::with_capacity(field_set.len());

        let doc: HashMap<Field, OwnedValue> = searcher
            .doc(addr)
            .map_err(|e| Status::internal(format!("Failed to retrieve document: {e}")))?;

        for &field_name in &field_set {
            let field = schema.get_idx(field_name)?;
            //.map_err(|e| {
            // Status::invalid_argument(format!("Invalid field name '{}': {}", field_name, e))
            // })?;

            let value = doc
                .get(&field)
                .map(|v| map_owned_value(field_name, v.clone()))
                .or_else(|| {
                    let col_type = index.schema.get_column(field_name).unwrap();
                    if col_type.is_nullable {
                        Some(match col_type.column_type {
                            MetaColumnType::Bool => NullableBool(None),
                            MetaColumnType::Ulong => NullableUlong(None),
                            MetaColumnType::Long => NullableLong(None),
                            MetaColumnType::Double => NullableDouble(None),
                            MetaColumnType::DateTime => NullableDateTime(None),
                            MetaColumnType::String => NullableStr(None),
                            MetaColumnType::Bytes => NullableBytes(None),
                            MetaColumnType::Tree => NullableTree(None),
                        })
                        .map(|sv| SearchField {
                            name: field_name.to_string(),
                            value: sv,
                        })
                    } else {
                        None
                    }
                })
                .ok_or(anyhow!("Unexpected null for {}", field_name))?;

            fields.push(value);
        }

        // for func in &req.functions {
        //     let program = parse_and_compile_program(func)?;
        //     let ctx = build_context(&program, &doc, schema)?;
        //     let result = evaluation::execute(&program, &ctx).map(|v| SearchField {
        //         name: func.to_string(),
        //         value: Double(v as f64),
        //     })?;
        //     fields.push(result);
        // }
        rows.push(api::Row { fields });
    }
    Ok(api::SearchResponse { rows })
}

fn parse_and_compile_program(func: &str) -> Result<Program> {
    let expr = Expr::parse(func).into_result().map_err(|errs| {
        anyhow!(
            "Failed to parse function: {}",
            errs.into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    Ok(Program::compile_expr(expr))
}

// fn build_context(
//     program: &Program,
//     doc: &HashMap<Field, OwnedValue>,
//     schema: &MetaSchema,
// ) -> Result<HashMap<String, f32>> {
//     program
//         .ops
//         .iter()
//         .filter_map(|op| match op {
//             OpCode::PushVariable(name) => Some(name.clone()),
//             _ => None,
//         })
//         .flat_map(|field_name| {
//             schema
//                 .get_idx(&field_name)
//                 .ok()
//                 .map(|field| (field_name, field))
//         })
//         .map(|(field_name, field)| {
//             doc.get(&field)
//                 .ok_or_else(|| anyhow!("Cannot handle null value for column: {field_name}"))
//                 .and_then(|v| owned_val_as_f32(v).map(|f| (field_name, f)))
//         })
//         .collect()
// }

// pub fn build_matrix_response(
//     index: &SearchIndex,
//     top_docs: &[(Score, DocAddress)],
// ) -> Result<SearchMatrixResponse, Status> {
//     let searcher = index.reader.searcher();
//     let schema = index.index.schema();

//     let mut matrix: HashMap<u32, Vec<OwnedValue>> = HashMap::with_capacity(schema.num_fields());
//     let init_vec = || Vec::with_capacity(top_docs.len());
//     for &(_, addr) in top_docs {
//         let doc: HashMap<Field, OwnedValue> = searcher
//             .doc(addr)
//             .map_err(|e| Status::internal(format!("Failed to retrieve doc: {e}")))?;

//         for (field, value) in doc {
//             matrix
//                 .entry(field.field_id())
//                 .or_insert_with(init_vec)
//                 .push(value);
//         }
//     }

//     let row_count = top_docs.len() as u32;

//     let columns = matrix
//         .into_iter()
//         .map(|(id, values)| {
//             let field = Field::from_field_id(id);
//             let name = schema.get_field_name(field).to_string();
//             values_to_column(name, values, &schema).expect("must be valid")
//         })
//         .collect();

//     Ok(SearchMatrixResponse { row_count, columns })
// }

// pub fn values_to_column(
//     name: String,
//     values: Vec<OwnedValue>,
//     schema: &Schema,
// ) -> Result<ColumnVector, Status> {
//     let field = schema
//         .get_field(&name)
//         .map_err(|_| Status::invalid_argument(format!("Field not in schema: {name}")))?;

//     let field_type = schema.get_field_entry(field).field_type();

//     let column = match field_type {
//         FieldType::Bool(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalBool {
//                     value: match v {
//                         OwnedValue::Bool(b) => Some(b),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Bools(BoolColumn { values })),
//             }
//         }
//         FieldType::U64(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalUInt64 {
//                     value: match v {
//                         OwnedValue::U64(u) => Some(u),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Ulongs(UInt64Column { values })),
//             }
//         }
//         FieldType::I64(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalInt64 {
//                     value: match v {
//                         OwnedValue::I64(i) => Some(i),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Longs(Int64Column { values })),
//             }
//         }
//         FieldType::F64(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalDouble {
//                     value: match v {
//                         OwnedValue::F64(f) => Some(f),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Doubles(DoubleColumn { values })),
//             }
//         }
//         FieldType::Str(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalString {
//                     value: match v {
//                         OwnedValue::Str(s) => Some(s),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Strings(StringColumn { values })),
//             }
//         }
//         FieldType::Bytes(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalBytes {
//                     value: match v {
//                         OwnedValue::Bytes(b) => Some(b),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Bytes(BytesColumn { values })),
//             }
//         }
//         FieldType::Facet(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalString {
//                     value: match v {
//                         OwnedValue::Facet(f) => Some(f.to_path_string()),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Facets(FacetColumn { values })),
//             }
//         }
//         FieldType::Date(_) => {
//             let values = values
//                 .into_iter()
//                 .map(|v| OptionalTimestampMs {
//                     value: match v {
//                         OwnedValue::Date(dt) => Some(dt.into_timestamp_millis()),
//                         _ => None,
//                     },
//                 })
//                 .collect();
//             ColumnVector {
//                 name,
//                 values: Some(Values::Timestamps(TimestampColumn { values })),
//             }
//         }
//         FieldType::JsonObject(_) => todo!(),
//         FieldType::IpAddr(_) => todo!(),
//     };

//     Ok(column)
// }

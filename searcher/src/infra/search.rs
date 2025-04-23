use crate::domain::document::map_owned_value;
use crate::infra::index::SearchIndex;
use corelib::proto::searcher::{SearchField, *};
use std::collections::{HashMap, HashSet};
use tantivy::query::QueryParser;
use tantivy::{
    DocAddress, Score,
    collector::TopDocs,
    schema::{Field, OwnedValue},
};
use tonic::Status;

pub fn execute_search(
    index: &SearchIndex,
    query_str: &str,
) -> Result<Vec<(Score, DocAddress)>, Status> {
    let searcher = index.reader.searcher();
    let schema = index.index.schema();

    let default_fields = ["title", "description"]
        .iter()
        .filter_map(|&name| schema.get_field(name).ok())
        .collect::<Vec<_>>();

    let parser = QueryParser::for_index(&index.index, default_fields);
    let query = parser
        .parse_query(query_str)
        .map_err(|e| Status::invalid_argument(format!("Invalid query: {e}")))?;

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(10))
        .map_err(|e| Status::internal(format!("Search failed: {e}")))?;

    Ok(top_docs)
}

pub fn build_search_response(
    index: &SearchIndex,
    top_docs: &[(Score, DocAddress)],
    projection: &[&str],
) -> Result<SearchResponse, Status> {
    let searcher = index.reader.searcher();
    let schema = index.index.schema();

    let mut hits = Vec::with_capacity(top_docs.len());

    let field_set: HashSet<&str> = HashSet::from_iter(projection.iter().copied());

    for &(_, addr) in top_docs {
        let doc: HashMap<Field, OwnedValue> = searcher
            .doc(addr)
            .map_err(|e| Status::internal(format!("Failed to retrieve doc: {e}")))?;

        let fields = field_set
            .iter()
            .filter_map(|field_name| match schema.get_field(field_name) {
                Ok(field) => Some(
                    doc.get(&field)
                        .ok_or_else(|| {
                            Status::not_found(format!(
                                "Field '{}' not found, asin: {}",
                                field_name,
                                if let Some(OwnedValue::Str(s)) =
                                    doc.get(&schema.get_field("asin").unwrap())
                                {
                                    &s
                                } else {
                                    "unknown"
                                }
                            ))
                        })
                        .map(|value| map_owned_value(field_name, value.clone())),
                ),
                Err(e) => Some(Err(Status::invalid_argument(format!(
                    "Invalid field name '{}': {}",
                    field_name, e
                )))),
            })
            .collect::<Result<Vec<SearchField>, Status>>()?;

        hits.push(SearchHit { fields });
    }

    Ok(SearchResponse { hits })
}

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

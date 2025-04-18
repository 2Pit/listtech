// use anyhow::Result;
// use corelib::proto::searcher::{column_vector::Values::*, *};
// use tabled::{builder::Builder, settings::Style};

// /// Печатает таблицу на основе транспонированного SearchMatrixResponse
// pub fn print_matrix_table(matrix: SearchMatrixResponse) -> Result<()> {
//     let headers: Vec<String> = matrix.columns.iter().map(|c| c.name.clone()).collect();
//     let row_count = matrix.row_count as usize;

//     // Строим таблицу
//     let mut builder = Builder::default();
//     builder.push_record(headers);

//     matrix.columns.into_iter().for_each(|column| {
//         let values = extract_column_values(&column, row_count);
//         builder.push_column(values);
//     });

//     let mut table = builder.build();
//     table.with(Style::psql());
//     println!("{table}");

//     Ok(())
// }

// /// Форматирует значения из колонки в строковый вектор
// fn extract_column_values(column: &ColumnVector, row_count: usize) -> Vec<String> {
//     fn format_opt<T>(opt: Option<T>, f: impl FnOnce(T) -> String) -> String {
//         opt.map(f).unwrap_or_default()
//     }

//     match &column.values {
//         Some(Bools(BoolColumn { values })) => values
//             .iter()
//             .map(|v| format_opt(v.value, |b| b.to_string()))
//             .collect(),

//         Some(Longs(Int64Column { values })) => values
//             .iter()
//             .map(|v| format_opt(v.value, |i| i.to_string()))
//             .collect(),

//         Some(Ulongs(UInt64Column { values })) => values
//             .iter()
//             .map(|v| format_opt(v.value, |u| u.to_string()))
//             .collect(),

//         Some(Doubles(DoubleColumn { values })) => values
//             .iter()
//             .map(|v| format_opt(v.value, |f| f.to_string()))
//             .collect(),

//         Some(Strings(StringColumn { values })) => values
//             .iter()
//             .map(|v| match &v.value {
//                 Some(s) if s.is_empty() => r#""""#.to_string(),
//                 Some(s) if s.len() > 15 => {
//                     let preview = s.chars().take(12).collect::<String>();
//                     format!(r#""{}...""#, preview)
//                 }
//                 Some(s) => format!(r#""{}""#, s),
//                 None => "".to_string(),
//             })
//             .collect(),

//         Some(Bytes(BytesColumn { values })) => values
//             .iter()
//             .map(|v| match &v.value {
//                 Some(b) if b.is_empty() => String::from("[]"),
//                 Some(b) if b.len() > 15 => {
//                     let preview = &b[..12.min(b.len())];
//                     format!("{:?}...", preview)
//                 }
//                 Some(b) => format!("{:?}", b),
//                 None => "".to_string(),
//             })
//             .collect(),

//         Some(Timestamps(TimestampColumn { values })) => values
//             .iter()
//             .map(|v| format_opt(v.value, |t| format!("ts({})", t)))
//             .collect(),

//         Some(Facets(FacetColumn { values })) => values
//             .iter()
//             .map(|v| match &v.value {
//                 Some(s) if s.is_empty() => r#""""#.to_string(),
//                 Some(s) if s.len() > 15 => {
//                     let preview = s.chars().take(12).collect::<String>();
//                     format!(r#""{}...""#, preview)
//                 }
//                 Some(s) => format!(r#""{}""#, s),
//                 None => "".to_string(),
//             })
//             .collect(),

//         None => vec!["".to_string(); row_count],
//     }
// }

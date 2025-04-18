use anyhow::Result;
use searcher::api::proto::searcher::*;
use tabled::settings::Style;
use tabled::{Table, Tabled};

/// Строка таблицы (динамическая)
#[derive(Debug)]
struct Row {
    #[allow(dead_code)]
    index: usize,
    values: Vec<String>,
}

/// Печатает таблицу на основе SearchMatrixResponse
pub fn print_matrix_table(matrix: &SearchMatrixResponse) -> Result<()> {
    let headers: Vec<String> = matrix.columns.iter().map(|c| c.name.clone()).collect();
    let row_count = matrix.row_count as usize;

    // Восстановим значения построчно
    let mut rows: Vec<Row> = (0..row_count)
        .map(|i| Row {
            index: i + 1,
            values: Vec::new(),
        })
        .collect();

    for column in &matrix.columns {
        let values: Vec<String> = extract_column_values(column, row_count);
        for (i, v) in values.into_iter().enumerate() {
            rows[i].values.push(v);
        }
    }

    // Преобразуем в табличный формат
    #[derive(Tabled)]
    struct RenderRow {
        #[tabled(rename = "#")]
        index: usize,
        #[tabled(inline)]
        fields: Vec<String>,
    }

    let render_rows: Vec<RenderRow> = rows
        .into_iter()
        .map(|r| RenderRow {
            index: r.index,
            fields: r.values,
        })
        .collect();

    let table = Table::new(render_rows)
        .with(Style::psql())
        .with(tabled::Modify::new(tabled::Full).with(tabled::Alignment::left()));

    // Вставим заголовки вручную
    println!(
        "{}",
        table
            .to_string()
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 1 {
                    let titles = headers
                        .iter()
                        .map(|s| format!(" {:<10}", s))
                        .collect::<Vec<_>>()
                        .join("|");
                    format!("| {:<3} |{}|", "#", titles)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(())
}

/// Форматирует значения из колонки в строковый вектор
fn extract_column_values(column: &ColumnVector, row_count: usize) -> Vec<String> {
    use searcher::api::proto::google::protobuf::*;

    fn format_opt<T>(opt: Option<T>, f: impl FnOnce(T) -> String) -> String {
        opt.map(f).unwrap_or_default()
    }

    match &column.values {
        Some(column_vector::Values::BoolColumn(BoolColumn { values })) => values
            .iter()
            .map(|v| format_opt(v.value, |b| b.to_string()))
            .collect(),

        Some(column_vector::Values::LongColumn(Int64Column { values })) => values
            .iter()
            .map(|v| format_opt(v.value, |i| i.to_string()))
            .collect(),

        Some(column_vector::Values::UlongColumn(UInt64Column { values })) => values
            .iter()
            .map(|v| format_opt(v.value, |u| u.to_string()))
            .collect(),

        Some(column_vector::Values::DoubleColumn(DoubleColumn { values })) => values
            .iter()
            .map(|v| format_opt(v.value, |f| f.to_string()))
            .collect(),

        Some(column_vector::Values::StringColumn(StringColumn { values })) => values
            .iter()
            .map(|v| match &v.value {
                Some(s) if s.is_empty() => r#""""#.to_string(),
                Some(s) => format!(r#""{}""#, s),
                None => "".to_string(),
            })
            .collect(),

        Some(column_vector::Values::BytesColumn(BytesColumn { values })) => values
            .iter()
            .map(|v| format_opt(v.value.clone(), |b| format!("{:?}", b)))
            .collect(),

        Some(column_vector::Values::TimestampColumn(TimestampColumn { values })) => values
            .iter()
            .map(|v| format_opt(v.value, |t| format!("ts({})", t)))
            .collect(),

        Some(column_vector::Values::FacetColumn(FacetColumn { values })) => values
            .iter()
            .map(|FacetWrapper { facets }| {
                if facets.is_empty() {
                    "".to_string()
                } else {
                    format!("[{}]", facets.join(", "))
                }
            })
            .collect(),

        None => vec!["".to_string(); row_count],
    }
}

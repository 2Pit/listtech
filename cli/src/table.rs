use chrono::DateTime;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use corelib::proto::searcher::{SearchField, SearchResponse, search_field::Value};
use std::collections::{BTreeMap, BTreeSet};

fn format_date(ts_millis: i64) -> String {
    DateTime::from_timestamp_millis(ts_millis)
        .map(|date_time| date_time.date_naive())
        .map(|native| format!("{}", native))
        .unwrap_or_else(|| "err".to_string())
}

pub fn print_results(resp: &SearchResponse) -> anyhow::Result<()> {
    if resp.hits.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    // Собираем все имена колонок
    let mut all_field_names = BTreeSet::new();
    for hit in &resp.hits {
        for field in &hit.fields {
            all_field_names.insert(field.name.clone());
        }
    }

    // Создаём таблицу
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    // .set
    // .set_width(140); // Или 100, или динамически можно читать ширину терминала

    // Заголовки
    let mut headers = vec![Cell::new("#")];
    headers.extend(all_field_names.iter().map(|name| Cell::new(name)));
    table.set_header(headers);

    // Данные
    for (i, hit) in resp.hits.iter().enumerate() {
        let mut field_map: BTreeMap<String, String> = BTreeMap::new();

        for SearchField { name, value } in &hit.fields {
            let val_str = match value {
                Some(Value::StringValue(s)) => {
                    format!("\"{}\"", s.chars().take(10).collect::<String>())
                }
                Some(Value::DoubleValue(f)) => format!("{f:.2}"),
                Some(Value::UlongValue(u)) => u.to_string(),
                Some(Value::LongValue(i)) => i.to_string(),
                Some(Value::BoolValue(b)) => b.to_string(),
                Some(Value::BytesValue(b)) => format!("{:?}", b),
                Some(Value::TimestampMsValue(ts)) => format_date(*ts),
                Some(Value::FacetWrapper(f)) => {
                    format!(
                        "[{}]",
                        f.facets
                            .iter()
                            .map(|facet| format!("\"{}\", ", facet.to_string()))
                            .collect::<String>()
                    )
                }
                None => "".to_string(),
            };
            field_map.insert(name.clone(), val_str);
        }

        let mut row = vec![Cell::new((i + 1).to_string())];
        for name in &all_field_names {
            row.push(Cell::new(field_map.get(name).cloned().unwrap_or_default()));
        }

        table.add_row(row);
    }

    println!("{table}");

    Ok(())
}

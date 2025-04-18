use corelib::proto::searcher::SearchResponse;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Row {
    id: String,
    title: String,
    price: String,
}

pub fn print_results(resp: &SearchResponse) -> anyhow::Result<()> {
    let rows: Vec<Row> = resp
        .hits
        .iter()
        .map(|doc| Row {
            id: doc.id.clone(),
            title: doc.fields.get("title").cloned().unwrap_or_default(),
            price: doc.fields.get("price").cloned().unwrap_or_default(),
        })
        .collect();

    let table = Table::new(rows);
    println!("{table}");

    Ok(())
}

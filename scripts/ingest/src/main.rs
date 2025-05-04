use anyhow::{Context, Result};
use corelib::telemetry::init::{init_logging, read_env_var};
use fields_builder::FieldsBuilder;
use indexer::api;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod fields_builder;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let api_port: u16 = read_env_var("INDEXER_HTTP_PORT", None)?;
    let api_addr = format!("http://localhost:{}", api_port);
    let client = reqwest::Client::new();

    let file = File::open("data/meta_Electronics.json").context("cannot open input file")?;
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        // if i >= 1000 {
        // break;
        // }

        let line = line?;
        let json: serde_json::Value =
            serde_json::from_str(&line).with_context(|| format!("invalid JSON at line {i}"))?;

        let doc = api::Document {
            index_name: "electronics".to_string(),
            index_version: 1,
            fields: map_json_to_fields(&json),
        };

        let res = client
            .post(format!("{}/v1/doc", api_addr))
            .json(&doc)
            .send()
            .await
            .with_context(|| format!("failed to send document at line {i}"))?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await.unwrap_or_default();
            tracing::error!(line = i, status = %status, body = %text, "indexing failed");
        } else {
            tracing::info!(line = i, "indexed");
        }
    }

    Ok(())
}

fn map_json_to_fields(json: &serde_json::Value) -> Vec<api::IndexableField> {
    FieldsBuilder::new(json)
        .string("asin")
        .string("title")
        .string("main_cat")
        .string("brand_string")
        .string("tech1")
        .string("tech2")
        // .string("similar_item")
        .string("image_url")
        .string("image_url_high_res")
        .first_string_from_array_or_empty("description")
        .first_string_from_array_or_empty("feature")
        .price()
        .date()
        .brand_facet()
        .category_facet()
        .rank()
        .build()
}

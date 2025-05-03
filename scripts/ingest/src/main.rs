use anyhow::{Context, Result};
use chrono::NaiveDate;
use corelib::telemetry::init::{init_logging, read_env_var};
use indexer::api;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
    let mut fields = Vec::new();

    macro_rules! add_string {
        ($name:expr) => {
            if let Some(s) = json.get($name).and_then(|v| v.as_str()) {
                fields.push(api::IndexableField {
                    name: $name.to_string(),
                    value: Some(api::FieldValue::String(s.to_string())),
                });
            }
        };
    }

    macro_rules! add_optional_string_array_first {
        ($name:expr) => {
            let val = json
                .get($name)
                .and_then(|v| v.as_array())
                .map(|arr| arr.first().and_then(|v| v.as_str()).unwrap_or(""))
                .unwrap_or("");

            fields.push(api::IndexableField {
                name: $name.to_string(),
                value: Some(api::FieldValue::String(val.to_string())),
            });
        };
    }

    add_string!("asin");
    add_string!("title");
    add_string!("main_cat");
    add_string!("brand_string");
    add_string!("tech1");
    add_string!("tech2");
    // add_string!("similar_item");
    add_string!("image_url");
    add_string!("image_url_high_res");

    add_optional_string_array_first!("description");
    add_optional_string_array_first!("feature");

    if let Some(price_str) = json.get("price").and_then(|v| v.as_str()) {
        if let Ok(price) = price_str.trim_start_matches('$').parse::<f64>() {
            fields.push(api::IndexableField {
                name: "price".to_string(),
                value: Some(api::FieldValue::Double(price)),
            });
        }
    }

    if let Some(date_str) = json.get("date").and_then(|v| v.as_str()) {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
            if let Some(ts) = date.and_hms_opt(0, 0, 0) {
                fields.push(api::IndexableField {
                    name: "timestamp_creation_ms".to_string(),
                    value: Some(api::FieldValue::DateTime(ts.format("%+").to_string())),
                });
            }
        }
    }

    if let Some(s) = json.get("brand").and_then(|v| v.as_str()) {
        fields.push(api::IndexableField {
            name: "brand".to_string(),
            value: Some(api::FieldValue::Tree(vec![format!("/{}", s)])),
        });
    }

    if let Some(arr) = json.get("category").and_then(|v| v.as_array()) {
        let path = arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join("/");

        if !path.is_empty() {
            fields.push(api::IndexableField {
                name: "category".to_string(),
                value: Some(api::FieldValue::Tree(vec![format!("/{}", path)])),
            });
        }
    }

    if let Some(arr) = json.get("rank").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(text) = item.as_str() {
                if let Some((rank_str, cat_str)) = text.split_once(" in ") {
                    let rank_clean = rank_str.trim_start_matches(">#").replace(",", "");
                    if let Ok(rank_value) = rank_clean.parse::<u64>() {
                        fields.push(api::IndexableField {
                            name: "rank_position".to_string(),
                            value: Some(api::FieldValue::Ulong(rank_value)),
                        });
                    }

                    let facet_path = format!(
                        "/{}",
                        cat_str.replace(" &gt; ", "/").replace(" > ", "/").trim()
                    );
                    fields.push(api::IndexableField {
                        name: "rank_facet".to_string(),
                        value: Some(api::FieldValue::Tree(vec![facet_path])),
                    });
                }
            }
        }
    }

    for f in &fields {
        if let Some(api::FieldValue::String(ref s)) = f.value {
            if s.len() > 65530 {
                tracing::warn!(field = %f.name, len = s.len(), value = %s, "String too long");
            }
        }
        if let Some(api::FieldValue::Tree(ref paths)) = f.value {
            for path in paths {
                if path.len() > 65530 {
                    tracing::warn!(field = %f.name, len = path.len(), value = %path, "Facet path too long");
                }
            }
        }
    }

    fields
}

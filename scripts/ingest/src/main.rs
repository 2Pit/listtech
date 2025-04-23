use anyhow::{Context, Result};
use chrono::NaiveDate;
use corelib::proto::indexer::{
    AddDocumentRequest, Document, IndexableField, indexable_field::FacetWrapper,
    indexable_field::Value, indexer_api_client::IndexerApiClient,
};
use corelib::telemetry::init::{init_logging, read_env_var};
use std::fs::File;
use std::io::{BufRead, BufReader};
use tonic::Request;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let port: u16 = read_env_var("INDEXER_GRPC_PORT", None)?;
    let grpc_addr = format!("http://localhost:{}", port);
    let mut client = IndexerApiClient::connect(grpc_addr).await?;

    let file = File::open("data/meta_Electronics.json").context("cannot open input file")?;
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        // if i >= 1000 {
        // break;
        // }

        let line = line?;
        let json: serde_json::Value =
            serde_json::from_str(&line).with_context(|| format!("invalid JSON at line {i}"))?;

        let doc = Document {
            schema_version: "v1".to_string(),
            fields: map_json_to_fields(&json),
        };

        let request = Request::new(AddDocumentRequest {
            document: Some(doc),
        });

        match client.add_document(request).await {
            Ok(_) => info!(line = i, "document indexed"),
            Err(err) => error!(line = i, error = %err, "failed to index"),
        }
    }

    Ok(())
}

fn map_json_to_fields(json: &serde_json::Value) -> Vec<IndexableField> {
    let mut fields = Vec::new();

    macro_rules! add_string {
        ($name:expr) => {
            if let Some(s) = json.get($name).and_then(|v| v.as_str()) {
                fields.push(IndexableField {
                    name: $name.to_string(),
                    value: Some(Value::StringValue(s.to_string())),
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

            fields.push(IndexableField {
                name: $name.to_string(),
                value: Some(Value::StringValue(val.to_string())),
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
            fields.push(IndexableField {
                name: "price".to_string(),
                value: Some(Value::DoubleValue(price)),
            });
        }
    }

    if let Some(date_str) = json.get("date").and_then(|v| v.as_str()) {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
            if let Some(ts) = date.and_hms_opt(0, 0, 0) {
                fields.push(IndexableField {
                    name: "timestamp_creation_ms".to_string(),
                    value: Some(Value::TimestampMsValue(ts.and_utc().timestamp_millis())),
                });
            }
        }
    }

    if let Some(s) = json.get("brand").and_then(|v| v.as_str()) {
        fields.push(IndexableField {
            name: "brand".to_string(),
            value: Some(Value::FacetWrapper(FacetWrapper {
                facets: vec![format!("/{}", s)],
            })),
        });
    }

    if let Some(arr) = json.get("category").and_then(|v| v.as_array()) {
        let path = arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join("/");

        if !path.is_empty() {
            fields.push(IndexableField {
                name: "category".to_string(),
                value: Some(Value::FacetWrapper(FacetWrapper {
                    facets: vec![format!("/{}", path)],
                })),
            });
        }
    }

    if let Some(arr) = json.get("rank").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(text) = item.as_str() {
                if let Some((rank_str, cat_str)) = text.split_once(" in ") {
                    let rank_clean = rank_str.trim_start_matches(">#").replace(",", "");
                    if let Ok(rank_value) = rank_clean.parse::<u64>() {
                        fields.push(IndexableField {
                            name: "rank_position".to_string(),
                            value: Some(Value::UlongValue(rank_value)),
                        });
                    }

                    let facet_path = format!(
                        "/{}",
                        cat_str.replace(" &gt; ", "/").replace(" > ", "/").trim()
                    );
                    fields.push(IndexableField {
                        name: "rank_facet".to_string(),
                        value: Some(Value::FacetWrapper(FacetWrapper {
                            facets: vec![facet_path],
                        })),
                    });
                }
            }
        }
    }

    for f in &fields {
        if let Some(Value::StringValue(ref s)) = f.value {
            if s.len() > 65530 {
                tracing::warn!(field = %f.name, len = s.len(), value = %s, "StringValue too long");
            }
        }
        if let Some(Value::FacetWrapper(ref facet)) = f.value {
            for facet_path in &facet.facets {
                if facet_path.len() > 65530 {
                    tracing::warn!(field = %f.name, len = facet_path.len(), value = %facet_path, "Facet path too long");
                }
            }
        }
    }

    fields
}

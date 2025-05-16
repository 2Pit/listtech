use chrono::{NaiveDate, NaiveDateTime};
use indexer::api;
use rand::Rng;

pub struct FieldsBuilder<'a> {
    json: &'a serde_json::Value,
    fields: Vec<api::IndexableField>,
}

impl<'a> FieldsBuilder<'a> {
    pub fn new(json: &'a serde_json::Value) -> Self {
        Self {
            json,
            fields: Vec::new(),
        }
    }

    pub fn string(mut self, name: &str) -> Self {
        if let Some(s) = self.json.get(name).and_then(|v| v.as_str()) {
            self.fields.push(api::IndexableField {
                name: name.to_string(),
                value: Some(api::FieldValue::Text(s.to_string())),
            });
        }
        self
    }

    pub fn first_string_from_array_or_empty(mut self, name: &str) -> Self {
        let val = self
            .json
            .get(name)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first().and_then(|v| v.as_str()))
            .unwrap_or("");
        self.fields.push(api::IndexableField {
            name: name.to_string(),
            value: Some(api::FieldValue::Text(val.to_string())),
        });
        self
    }

    pub fn price(mut self) -> Self {
        let maybe_price = self
            .json
            .get("price")
            .and_then(|v| v.as_str())
            .and_then(|s| s.trim_start_matches('$').parse::<f64>().ok());

        let price = maybe_price.unwrap_or_else(|| {
            let mut rng = rand::rng();
            rng.random_range(1.0..=50.0)
        });

        self.fields.push(api::IndexableField {
            name: "price".to_string(),
            value: Some(api::FieldValue::Double(price)),
        });

        self
    }

    pub fn date(mut self) -> Self {
        let maybe_date = self
            .json
            .get("date")
            .and_then(|v| v.as_str())
            .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%B %d, %Y").ok())
            .and_then(|date| date.and_hms_opt(0, 0, 0));

        let ts = match maybe_date {
            Some(dt) => dt,
            None => {
                // Диапазон: 2018-01-01 00:00:00 ... 2019-01-01 00:00:00
                let start = NaiveDate::from_ymd_opt(2018, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp();
                let end = NaiveDate::from_ymd_opt(2019, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp();

                let mut rng = rand::rng();
                let random_ts = rng.random_range(start..end);
                NaiveDateTime::from_timestamp_opt(random_ts, 0).unwrap()
            }
        };

        self.fields.push(api::IndexableField {
            name: "timestamp_creation_ms".to_string(),
            value: Some(api::FieldValue::DateTime(
                ts.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            )),
        });

        self
    }

    pub fn brand_facet(mut self) -> Self {
        if let Some(s) = self.json.get("brand").and_then(|v| v.as_str()) {
            self.fields.push(api::IndexableField {
                name: "brand".to_string(),
                value: Some(api::FieldValue::Tree(vec![format!("/{}", s)])),
            });
        }
        self
    }

    pub fn category_facet(mut self) -> Self {
        if let Some(arr) = self.json.get("category").and_then(|v| v.as_array()) {
            let path = arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("/");

            if !path.is_empty() {
                self.fields.push(api::IndexableField {
                    name: "category".to_string(),
                    value: Some(api::FieldValue::Tree(vec![format!("/{}", path)])),
                });
            }
        }
        self
    }

    pub fn rank(mut self) -> Self {
        if let Some(arr) = self.json.get("rank").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(text) = item.as_str() {
                    if let Some((rank_str, cat_str)) = text.split_once(" in ") {
                        let rank_clean = rank_str.trim_start_matches(">#").replace(",", "");
                        if let Ok(rank_value) = rank_clean.parse::<u64>() {
                            self.fields.push(api::IndexableField {
                                name: "rank_position".to_string(),
                                value: Some(api::FieldValue::Ulong(rank_value)),
                            });
                        }

                        let facet_path = format!(
                            "/{}",
                            cat_str.replace(" &gt; ", "/").replace(" > ", "/").trim()
                        );
                        self.fields.push(api::IndexableField {
                            name: "rank_facet".to_string(),
                            value: Some(api::FieldValue::Tree(vec![facet_path])),
                        });
                    }
                }
            }
        }
        self
    }

    pub fn build(self) -> Vec<api::IndexableField> {
        for f in &self.fields {
            if let Some(api::FieldValue::Text(ref s)) = f.value {
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
        self.fields
    }
}

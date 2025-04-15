use indexer::api::mapping::map_proto_to_tantivy_doc;
use indexer::api::proto::indexer::indexable_field::Value;
use indexer::api::proto::indexer::{Document, IndexableField};
use indexer::indexing::schema::build_schema;
use tantivy::Index;

#[tokio::test]
async fn test_indexing_example_document() {
    let schema = build_schema();
    let index = Index::create_in_ram(schema.clone());
    let mut writer = index.writer(50_000_000).expect("writer init failed");

    let doc = Document {
        schema_version: "v1".to_string(),
        fields: vec![
            IndexableField {
                name: "asin".to_string(),
                value: Some(Value::StringValue("0011300000".to_string())),
                facets: vec![],
            },
            IndexableField {
                name: "title".to_string(),
                value: Some(Value::StringValue("Genuine Geovision 1 Channel 3rd Party NVR IP Software with USB Dongle Onvif PSIA".to_string())),
                facets: vec![],
            },
            IndexableField {
                name: "description".to_string(),
                value: Some(Value::StringValue("The following camera brands...".to_string())),
                facets: vec![],
            },
            IndexableField {
                name: "feature".to_string(),
                value: Some(Value::StringValue("Support 3rd Party IP Camera".to_string())),
                facets: vec![],
            },
            IndexableField {
                name: "price".to_string(),
                value: Some(Value::DoubleValue(65.0)),
                facets: vec![],
            },
            IndexableField {
                name: "main_cat".to_string(),
                value: Some(Value::StringValue("Camera & Photo".to_string())),
                facets: vec![],
            },
            IndexableField {
                name: "brand".to_string(),
                value: None,
                facets: vec!["/GeoVision".to_string()],
            },
            IndexableField {
                name: "brand_string".to_string(),
                value: Some(Value::StringValue("GeoVision".to_string())),
                facets: vec![],
            },
            IndexableField {
                name: "category".to_string(),
                value: None,
                facets: vec![
                    "/Electronics".to_string(),
                    "/Electronics/Camera & Photo".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance/Surveillance Systems".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance/Surveillance Systems/Surveillance DVR Kits".to_string(),
                ],
            },
            IndexableField {
                name: "rank_position".to_string(),
                value: Some(Value::UlongValue(3092)),
                facets: vec![],
            },
            IndexableField {
                name: "rank_facet".to_string(),
                value: None,
                facets: vec![
                    "/Tools & Home Improvement".to_string(),
                    "/Tools & Home Improvement/Safety & Security".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance/Complete Surveillance Systems".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance/Complete Surveillance Systems/Surveillance DVR Kits".to_string(),
                ],
            },
            IndexableField {
                name: "timestamp_creation_ms".to_string(),
                value: Some(Value::TimestampMsValue(1390876800000)),
                facets: vec![],
            },
        ],
    };

    let tantivy_doc = map_proto_to_tantivy_doc(&doc, &schema).expect("mapping failed");

    writer
        .add_document(tantivy_doc)
        .expect("add_document failed");
    writer.commit().expect("commit failed");
}

#[tokio::test]
async fn test_invalid_type_field() {
    let schema = build_schema();
    let doc = Document {
        schema_version: "v1".to_string(),
        fields: vec![IndexableField {
            name: "price".to_string(),
            value: Some(Value::StringValue("should_be_f64".to_string())), // Ошибка: ожидается f64
            facets: vec![],
        }],
    };

    let result = map_proto_to_tantivy_doc(&doc, &schema);
    assert!(
        result.is_err(),
        "Expected error when passing incorrect value type"
    );
}

#[tokio::test]
async fn test_unknown_field_name() {
    let schema = build_schema();
    let doc = Document {
        schema_version: "v1".to_string(),
        fields: vec![IndexableField {
            name: "unknown_field".to_string(), // Ошибка: поля нет в схеме
            value: Some(Value::BoolValue(true)),
            facets: vec![],
        }],
    };

    let result = map_proto_to_tantivy_doc(&doc, &schema);
    assert!(
        result.is_err(),
        "Expected error when using field not present in schema"
    );
}

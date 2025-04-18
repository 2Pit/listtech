use corelib::proto::indexer::indexable_field::FacetWrapper as FacetWrapperStruct;
use corelib::proto::indexer::indexable_field::Value::*;
use corelib::proto::indexer::{Document, IndexableField};
use indexer::api::mapping::map_proto_to_tantivy_doc;
use indexer::infra::schema::build_schema;
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
                value: Some(StringValue("0011300000".to_string())),
            },
            IndexableField {
                name: "title".to_string(),
                value: Some(StringValue("Genuine Geovision 1 Channel 3rd Party NVR IP Software with USB Dongle Onvif PSIA".to_string())),
            },
            IndexableField {
                name: "description".to_string(),
                value: Some(StringValue("The following camera brands...".to_string())),
            },
            IndexableField {
                name: "feature".to_string(),
                value: Some(StringValue("Support 3rd Party IP Camera".to_string())),
            },
            IndexableField {
                name: "price".to_string(),
                value: Some(DoubleValue(65.0)),
            },
            IndexableField {
                name: "main_cat".to_string(),
                value: Some(StringValue("Camera & Photo".to_string())),
            },
            IndexableField {
                name: "brand".to_string(),
                value: Some(FacetWrapper(FacetWrapperStruct{facets: vec!["/GeoVision".to_string()]})),
            },
            IndexableField {
                name: "brand_string".to_string(),
                value: Some(StringValue("GeoVision".to_string())),
            },
            IndexableField {
                name: "category".to_string(),
                value: Some(FacetWrapper(FacetWrapperStruct{facets: vec![
                    "/Electronics".to_string(),
                    "/Electronics/Camera & Photo".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance/Surveillance Systems".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance/Surveillance Systems/Surveillance DVR Kits".to_string(),
                ]})),
            },
            IndexableField {
                name: "rank_position".to_string(),
                value: Some(UlongValue(3092)),
            },
            IndexableField {
                name: "rank_facet".to_string(),
                value: Some(FacetWrapper(FacetWrapperStruct{facets: vec![
                    "/Tools & Home Improvement".to_string(),
                    "/Tools & Home Improvement/Safety & Security".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance/Complete Surveillance Systems".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance/Complete Surveillance Systems/Surveillance DVR Kits".to_string(),
                ]})),
            },
            IndexableField {
                name: "timestamp_creation_ms".to_string(),
                value: Some(TimestampMsValue(1390876800000)),
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
            value: Some(StringValue("should_be_f64".to_string())), // Ошибка: ожидается f64
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
            value: Some(BoolValue(true)),
        }],
    };

    let result = map_proto_to_tantivy_doc(&doc, &schema);
    assert!(
        result.is_err(),
        "Expected error when using field not present in schema"
    );
}

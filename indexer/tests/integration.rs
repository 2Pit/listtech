mod print_schema;
use indexer::api;
use indexer::api::*;
use indexer::infra::index::IndexState;
use tokio::sync::OnceCell;

static INDEX_STATE: OnceCell<IndexState> = OnceCell::const_new();

async fn build_index() -> &'static IndexState {
    INDEX_STATE
        .get_or_init(|| async {
            IndexState::init_index("../data/testing_index")
                .await
                .expect("Failed to initialize index")
        })
        .await
}

#[tokio::test]
async fn test_indexing_example_document() {
    let index_state = build_index().await;

    let api_doc = api::Document {
        index_name: "index_name".to_string(),
        index_version: 1,
        fields: vec![
            IndexableField {
                name: "asin".to_string(),
                value: Some(FieldValue::String("0011300000".to_string())),
            },
            IndexableField {
                name: "title".to_string(),
                value: Some(FieldValue::String("Genuine Geovision 1 Channel 3rd Party NVR IP Software with USB Dongle Onvif PSIA".to_string())),
            },
            IndexableField {
                name: "description".to_string(),
                value: Some(FieldValue::String("The following camera brands...".to_string())),
            },
            IndexableField {
                name: "feature".to_string(),
                value: Some(FieldValue::String("Support 3rd Party IP Camera".to_string())),
            },
            IndexableField {
                name: "price".to_string(),
                value: Some(FieldValue::Double(65.0)),
            },
            IndexableField {
                name: "main_cat".to_string(),
                value: Some(FieldValue::String("Camera & Photo".to_string())),
            },
            IndexableField {
                name: "brand".to_string(),
                value: Some(FieldValue::Tree(vec!["/GeoVision".to_string()])),
            },
            IndexableField {
                name: "brand_string".to_string(),
                value: Some(FieldValue::String("GeoVision".to_string())),
            },
            IndexableField {
                name: "category".to_string(),
                value: Some(FieldValue::Tree(vec![
                    "/Electronics".to_string(),
                    "/Electronics/Camera & Photo".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance/Surveillance Systems".to_string(),
                    "/Electronics/Camera & Photo/Video Surveillance/Surveillance Systems/Surveillance DVR Kits".to_string(),
                ])),
            },
            IndexableField {
                name: "rank_position".to_string(),
                value: Some(FieldValue::Ulong(3092)),
            },
            IndexableField {
                name: "rank_facet".to_string(),
                value: Some(FieldValue::Tree(vec![
                    "/Tools & Home Improvement".to_string(),
                    "/Tools & Home Improvement/Safety & Security".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance/Complete Surveillance Systems".to_string(),
                    "/Tools & Home Improvement/Safety & Security/Home Security & Surveillance/Complete Surveillance Systems/Surveillance DVR Kits".to_string(),
                ])),
            },
            IndexableField {
                name: "timestamp_creation_ms".to_string(),
                value: Some(FieldValue::DateTime("2025-04-26T18:19:03+00:00".to_string())),
            },
        ],
    };

    index_state
        .add_document_safely(api_doc)
        .await
        .expect("add_document failed");
}

#[tokio::test]
async fn test_invalid_type_field() {
    let index_state = build_index().await;
    let doc = Document {
        index_name: "some_name".to_string(),
        index_version: 1,
        fields: vec![IndexableField {
            name: "price".to_string(),
            value: Some(FieldValue::String("should_be_f64".to_string())), // Ошибка: ожидается f64
        }],
    };

    let result = index_state.add_document_safely(doc).await;
    assert!(
        result.is_err(),
        "Expected error when passing incorrect value type"
    );
}

#[tokio::test]
async fn test_unknown_field_name() {
    let index_state = build_index().await;
    let doc = Document {
        index_name: "some_name".to_string(),
        index_version: 1,
        fields: vec![IndexableField {
            name: "unknown_field".to_string(), // Ошибка: поля нет в схеме
            value: Some(FieldValue::Bool(true)),
        }],
    };

    let result = index_state.add_document_safely(doc).await;
    assert!(
        result.is_err(),
        "Expected error when using field not present in schema"
    );
}

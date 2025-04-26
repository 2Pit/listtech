use indexer::api;
use indexer::infra::schema::{InnerColumnType, InnerSchema};
use tantivy::schema::Field as TanField;

#[tokio::test]
async fn print_schema() {
    let field_descriptions = vec![
        ("asin", api::ColumnType::String),
        ("title", api::ColumnType::String),
        ("description", api::ColumnType::String),
        ("timestamp_creation_ms", api::ColumnType::DateTime),
        ("feature", api::ColumnType::String),
        ("main_cat", api::ColumnType::String),
        ("also_buy", api::ColumnType::String),
        ("also_view", api::ColumnType::String),
        ("image_url", api::ColumnType::String),
        ("image_url_high_res", api::ColumnType::String),
        ("tech1", api::ColumnType::String),
        ("tech2", api::ColumnType::String),
        ("similar_item", api::ColumnType::String),
        ("brand_string", api::ColumnType::String),
        ("brand", api::ColumnType::Tree),
        ("category", api::ColumnType::Tree),
        ("price", api::ColumnType::Double),
        ("rank_position", api::ColumnType::Ulong),
        ("rank_facet", api::ColumnType::Tree),
    ];

    let columns: Vec<InnerColumnType> = field_descriptions
        .iter()
        .enumerate()
        .map(|(i, (name, col_type))| InnerColumnType {
            tan_field: TanField::from_field_id(i as u32),
            field_type: col_type.clone(),
            name: name.to_string(),
            is_stored: true,
            is_eq: false,
            is_fast: matches!(col_type, api::ColumnType::Double | api::ColumnType::Ulong),
            is_tree: matches!(col_type, api::ColumnType::Tree),
            is_nullable: false,
        })
        .collect();

    let schema = InnerSchema {
        name: "products".to_string(),
        version: 1,
        id_column: columns[0].clone(),
        column_by_name: columns
            .iter()
            .map(|col| (col.name.clone(), col.clone()))
            .collect(),
        columns,
    };
    let json = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", json);
}

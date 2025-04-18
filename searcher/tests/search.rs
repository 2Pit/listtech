use corelib::proto::searcher::SearchHit;
use searcher::infra::index::SearchIndex;
use searcher::infra::search::execute_search;

use std::path::PathBuf;
use tantivy::schema::*;
use tantivy::{Index, doc};

fn create_test_index(dir: PathBuf) -> SearchIndex {
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let schema = schema_builder.build();

    let index = Index::create_in_dir(&dir, schema.clone()).unwrap();
    let mut writer = index.writer(50_000_000).unwrap();

    writer.add_document(doc!(title => "macbook pro")).unwrap();
    writer.add_document(doc!(title => "iphone 12")).unwrap();
    writer.commit().unwrap();

    let reader = index.reader().unwrap();

    SearchIndex { index, reader }
}

#[test]
fn test_search_finds_macbook() {
    let tmp = tempfile::tempdir().unwrap();
    let index = create_test_index(tmp.path().to_path_buf());

    let hits: Vec<SearchHit> = execute_search(&index, "macbook").unwrap();
    assert_eq!(hits.len(), 1);
    let hit = &hits[0];

    assert!(
        hit.fields
            .iter()
            .flat_map(|sf| sf.value.as_ref())
            .any(|v| match v {
                corelib::proto::searcher::search_field::Value::StringValue(s) =>
                    s.contains("macbook"),
                _ => false,
            })
    );
}

#[test]
fn test_search_unknown_word() {
    let tmp = tempfile::tempdir().unwrap();
    let index = create_test_index(tmp.path().to_path_buf());

    let hits: Vec<SearchHit> = execute_search(&index, "nobody").unwrap();
    assert_eq!(hits.len(), 0);
}

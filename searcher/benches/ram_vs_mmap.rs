use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use searcher::infra::{
    index::SearchIndex,
    search::{build_search_response, execute_search},
};
use tonic::Status;

fn search_benchmark(c: &mut Criterion) {
    let query_str = "Mac Book USB Apple HP LENOVO black box category:/Electronics";
    let index_dir = "../data/index";
    let index = SearchIndex::open_from_path(Path::new(&index_dir)).unwrap();
    // let index = SearchIndex::open_from_path_to_ram(Path::new(&index_dir)).unwrap();

    let projection = vec!["title", "description", "asin"];

    c.bench_function("search top 100", |b| {
        b.iter(|| {
            let top_docs = execute_search(&index, query_str)
                .map_err(|e| Status::internal(format!("Search execution failed: {}", e)))
                .unwrap();

            let _ = build_search_response(&index, &top_docs, &projection)
                .map_err(|e| Status::internal(format!("Failed to build search response: {}", e)))
                .unwrap();
        })
    });
}

criterion_group!(benches, search_benchmark);
criterion_main!(benches);

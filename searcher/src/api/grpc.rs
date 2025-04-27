// use crate::infra::query_parser::*;
use crate::infra::search::execute_search;
use crate::infra::{index::SearchIndex, search::build_search_response};
use corelib::proto::searcher::{
    SearchMatrixResponse, SearchRequest, SearchResponse, search_service_server::SearchService,
};
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl SearchService for SearchIndex {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let request_inner = request.into_inner();
        let query_str = request_inner.query;
        let projection = request_inner.return_fields;

        let top_docs = execute_search(self, query_str.as_str())
            .map_err(|e| Status::internal(format!("Search execution failed: {}", e)))?;

        let projection_strs: Vec<&str> = projection.iter().map(|s| s.as_str()).collect();
        let response = build_search_response(self, &top_docs, &projection_strs)
            .map_err(|e| Status::internal(format!("Failed to build search response: {}", e)))?;

        Ok(Response::new(response))
    }

    async fn search_matrix(
        &self,
        _: Request<SearchRequest>,
    ) -> Result<Response<SearchMatrixResponse>, Status> {
        todo!()
    }
}

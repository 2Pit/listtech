use crate::infra::index::SearchIndex;
use crate::infra::search::*;
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
        let query_str = request.into_inner().query;
        let top_docs = execute_search(self, &query_str)?;
        let response = build_search_response(self, &top_docs)?;
        Ok(Response::new(response))
    }

    async fn search_matrix(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchMatrixResponse>, Status> {
        let query_str = request.into_inner().query;
        let top_docs = execute_search(self, &query_str)?;
        let response = build_matrix_response(self, &top_docs)?;
        Ok(Response::new(response))
    }
}

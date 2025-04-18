use crate::infra::index::SearchIndex;
use crate::infra::search::execute_search;
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
        let hits = execute_search(self, &query_str)?;
        Ok(Response::new(SearchResponse { hits }))
    }

    async fn search_matrix(
        &self,
        _request: Request<SearchRequest>,
    ) -> Result<Response<SearchMatrixResponse>, Status> {
        todo!()
    }
}

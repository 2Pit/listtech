use crate::api::proto::indexer::indexer_api_server::IndexerApi;
use crate::api::proto::indexer::*;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct IndexerGrpc {}

#[tonic::async_trait]
impl IndexerApi for IndexerGrpc {
    async fn add_document(
        &self,
        request: Request<AddDocumentRequest>,
    ) -> Result<Response<AddDocumentResponse>, Status> {
        let doc = request.into_inner().document;
        tracing::info!(?doc, "Received document");

        Ok(Response::new(AddDocumentResponse {}))
    }

    async fn add_documents(
        &self,
        _request: Request<AddDocumentsRequest>,
    ) -> Result<Response<AddDocumentsResponse>, Status> {
        todo!()
    }

    async fn update_schema(
        &self,
        _request: Request<UpdateSchemaRequest>,
    ) -> Result<Response<UpdateSchemaResponse>, Status> {
        todo!()
    }

    async fn get_schema(
        &self,
        _request: Request<GetSchemaRequest>,
    ) -> Result<Response<GetSchemaResponse>, Status> {
        todo!()
    }
}

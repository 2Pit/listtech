use crate::infra::index::IndexState;
use crate::infra::index_writer::index_document;
use crate::infra::schema::build_schema;
use corelib::proto::indexer::indexer_api_server::IndexerApi;
use corelib::proto::indexer::*;
use std::path::Path;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct IndexerGrpc {
    pub index: IndexState,
}

impl IndexerGrpc {
    pub async fn create_indexer_grpc(index_path: &Path) -> anyhow::Result<IndexerGrpc> {
        let schema = build_schema();
        let index_state = IndexState::init_index(index_path, schema).await?;
        Ok(IndexerGrpc { index: index_state })
    }
}

#[tonic::async_trait]
impl IndexerApi for IndexerGrpc {
    async fn add_document(
        &self,
        request: Request<AddDocumentRequest>,
    ) -> Result<Response<AddDocumentResponse>, Status> {
        let doc = request
            .into_inner()
            .document
            .ok_or_else(|| Status::invalid_argument("Document is missing"))?;

        index_document(&self.index, doc)
            .await
            .map_err(|e| Status::invalid_argument(format!("Document indexing failed: {e}")))?;

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

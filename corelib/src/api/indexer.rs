use serde::{Deserialize, Serialize};

use super::indexer_model::document::Document;
use super::indexer_model::schema::Schema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    pub document: Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaRequest {
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaResponse;

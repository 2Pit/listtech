use serde::{Deserialize, Serialize};

use crate::api::*;
use corelib::api;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    pub schema_name: String,
    pub document: Document,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddDocumentResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaRequest {
    pub schema: api::MetaSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSchemaRequest {
    pub schema_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSchemaResponse {
    pub schema: api::MetaSchema,
}

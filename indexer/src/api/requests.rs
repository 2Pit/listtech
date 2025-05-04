use serde::{Deserialize, Serialize};

use crate::api::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDocumentRequest {
    pub document: Document,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddDocumentResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchemaRequest {
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSchemaRequest {
    pub schema_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSchemaResponse {
    pub schema: Schema,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddSchemaResponse;

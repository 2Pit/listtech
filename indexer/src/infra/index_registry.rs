use std::sync::Arc;

use dashmap::DashMap;

use super::index::IndexState;

pub type IndexRegistry = Arc<DashMap<String, Arc<IndexState>>>;

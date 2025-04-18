pub mod google {
    pub mod protobuf {
        include!("google.protobuf.rs");
    }
    pub mod api {
        include!("google.api.rs");
    }
}
pub mod searcher;

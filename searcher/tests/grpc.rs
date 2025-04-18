use corelib::proto::searcher::{
    SearchRequest, search_service_client::SearchServiceClient,
    search_service_server::SearchServiceServer,
};
use searcher::infra::index::SearchIndex;

use std::net::SocketAddr;
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::oneshot;
use tonic::Request;
use tonic::transport::{Endpoint, Server};

use tantivy::schema::*;
use tantivy::{Index, doc};

fn create_test_index() -> SearchIndex {
    let dir = tempdir().unwrap();
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let schema = schema_builder.build();

    let index = Index::create_in_dir(&dir, schema.clone()).unwrap();
    let _50mb = 50_000_000;
    let mut writer = index.writer(_50mb).unwrap();

    writer.add_document(doc!(title => "macbook pro")).unwrap();
    writer.commit().unwrap();

    let reader = index.reader().unwrap();
    SearchIndex { index, reader }
}

#[tokio::test]
async fn test_grpc_search_macbook() {
    // 1. Создаём сервер и клиента
    let index = create_test_index();
    let svc = SearchServiceServer::new(index);

    // 2. Запускаем сервер в фоне на случайном порту
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap(); // порт 0 = автоназначение
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = async move {
        Server::builder()
            .add_service(svc)
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                async {
                    shutdown_rx.await.ok();
                },
            )
            .await
            .unwrap();
    };

    let _handle = tokio::spawn(server);

    // 3. Создаём gRPC клиент
    let endpoint = format!("http://{}", addr);
    let channel = Endpoint::from_shared(endpoint)
        .unwrap()
        .connect_timeout(Duration::from_secs(3))
        .connect()
        .await
        .unwrap();

    let mut client = SearchServiceClient::new(channel);

    // 4. Делаем запрос
    let response = client
        .search(Request::new(SearchRequest {
            query: "macbook".into(),
        }))
        .await
        .unwrap()
        .into_inner();

    // 5. Проверяем результат
    assert_eq!(response.hits.len(), 1);
    let fields = &response.hits[0].fields;
    assert!(
        fields
            .iter()
            .flat_map(|sf| sf.value.as_ref())
            .any(|v| match v {
                corelib::proto::searcher::search_field::Value::StringValue(s) =>
                    s.contains("macbook"),
                _ => false,
            })
    );

    // 6. Останавливаем сервер
    let _ = shutdown_tx.send(());
}

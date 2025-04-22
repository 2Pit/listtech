use corelib::proto::searcher::search_service_client::SearchServiceClient;
use tonic::transport::Channel;

pub async fn create_client(addr: &str) -> anyhow::Result<SearchServiceClient<Channel>> {
    let client = SearchServiceClient::connect(addr.to_string()).await?;
    Ok(client)
}

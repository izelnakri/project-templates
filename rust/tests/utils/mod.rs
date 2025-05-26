use wiremock::MockServer;
use github_user_fetcher::adapter::HttpAdapter;
use github_user_fetcher::server::HttpServer;
use tokio::{net::TcpStream, time::Duration};
use std::net::{SocketAddr, TcpListener};
use std::error::Error;

pub mod mock_github_api;

#[allow(dead_code)]
pub fn get_random_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind random port")
        .local_addr()
        .unwrap()
        .port()
}

/// Sets up the mock server and HTTP server, returning handles for test use.
#[allow(dead_code)]
pub async fn setup_server(
    port: u16,
) -> (
    MockServer,
    HttpAdapter,
    tokio::task::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>
) {
    let mock_server = mock_github_api::setup().await;
    let github_api_adapter = HttpAdapter::new(mock_server.uri());
    let http_server = HttpServer::new(port);
    let join_handle = start_server_with_adapter(http_server, github_api_adapter.clone());

    (mock_server, github_api_adapter, join_handle)
}

/// Starts the HttpServer using an adapter, in a background task.
#[allow(dead_code)]
pub fn start_server_with_adapter(
    http_server: HttpServer,
    github_api_adapter: HttpAdapter,
) -> tokio::task::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
    tokio::spawn(async move {
        http_server
            .run(github_api_adapter)
            .await
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })
    })
}

/// Waits for a port to be ready to accept connections.
#[allow(dead_code)]
pub async fn wait_for_port_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(1000);

    while start.elapsed() < timeout {
        if TcpStream::connect(addr).await.is_ok() {
            return true;
        }
        tokio::task::yield_now().await;
    }

    false
}

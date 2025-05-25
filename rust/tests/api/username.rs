// TODO: These tests are still flaky, probably smt to do with variable assignment
use github_user_fetcher::server::listen;
use std::net::TcpListener;
use tokio::task::JoinHandle;

use crate::utils::with_mock_server;

use wiremock::{MockServer, Mock, ResponseTemplate, matchers::{method, path}};

fn get_random_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind random port")
        .local_addr()
        .unwrap()
        .port()
}

async fn spawn_server(port: u16) -> JoinHandle<()> {
    // TODO: This hangs, how can I make sure server is started? already?
    tokio::spawn(async move { listen(port).await.unwrap(); })
}

#[tokio::test]
async fn test_integration_fetch_username_ok() {
    with_mock_server(|_mock| async {
        let port = get_random_port();
        spawn_server(port).await;

        let client = reqwest::Client::new();
        let url = format!("http://localhost:{port}/api/octocat");

        let resp = client.get(&url).send().await.expect("Request failed");
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = resp.json().await.expect("Invalid JSON response");
        assert_eq!(body["login"], "octocat");
        assert_eq!(body["name"], "The Octocat");
        assert_eq!(body["company"], "GitHub");
        assert_eq!(body["location"], "San Francisco");
    })
    .await;
}

#[tokio::test]
async fn test_integration_fetch_username_not_found() {
    with_mock_server(|mock_server| async move {
        Mock::given(method("GET"))
            .and(path("/users/notfound"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;
        
        let port = get_random_port();
        spawn_server(port).await;

        let client = reqwest::Client::new();
        let url = format!("http://localhost:{port}/api/notfound");

        let resp = client.get(&url).send().await.expect("Request failed");
        assert_eq!(resp.status(), 404);

        let body: serde_json::Value = resp.json().await.expect("Invalid JSON response");
        assert!(
            body["message"]
                .as_str()
                .unwrap_or_default()
                .contains("notfound"),
            "Expected error message to mention username"
        );
    })
    .await;
}

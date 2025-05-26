use crate::utils;

#[tokio::test]
async fn test_integration_fetch_username_ok() {
    let port = utils::get_random_port();

    utils::setup_server(port).await;

    let client = reqwest::Client::new();
    let url = format!("http://localhost:{port}/api/octocat");

    let resp = client.get(&url).send().await.expect("Request failed");
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Invalid JSON response");
    assert_eq!(body["login"], "octocat");
    assert_eq!(body["name"], "The Octocat");
    assert_eq!(body["company"], "GitHub");
    assert_eq!(body["location"], "San Francisco");
}

#[tokio::test]
async fn test_integration_fetch_username_not_found() {
    let port = utils::get_random_port();

    utils::setup_server(port).await;
    
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
}

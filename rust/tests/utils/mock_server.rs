use wiremock::{MockServer, Mock, ResponseTemplate, matchers::{method, path}};

pub async fn setup() -> MockServer {
    let mock_server = MockServer::start().await;
    let user_json = r#"
    {
        "login": "octocat",
        "name": "The Octocat",
        "company": "GitHub",
        "location": "San Francisco"
    }
    "#;

    Mock::given(method("GET"))
        .and(path("/users/octocat"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(user_json, "application/json"))
        .mount(&mock_server)
        .await;

    return mock_server;
}

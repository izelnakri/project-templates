use wiremock::{MockServer, Mock, ResponseTemplate, matchers::{method, path}};

#[cfg(test)]
#[path = "../tests/utils/network_mock_test_case.rs"]
mod network_mock_test_case;

#[cfg(test)]
#[path = "../tests/utils/mock_server.rs"]
mod mock_server;

#[cfg(test)]
pub async fn with_mock_server<F, Fut, R>(test_fn: F) -> R 
where
    F: FnOnce(MockServer) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    network_mock_test_case::NetworkMockTestCase::register();
    let mock_server = mock_server::setup().await;

    crate::user::set_base_url_override(Some(mock_server.uri()));

    let result = test_fn(mock_server).await;

    crate::user::set_base_url_override(None);

    result
}

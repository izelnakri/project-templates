use wiremock::{MockServer};
use std::sync::{Mutex, MutexGuard};

pub mod mock_server;
pub mod network_mock_test_case;

/// Convenience macro for tests that need to mock network calls
/// 
/// # Example
/// ```rust
/// use crate::test_utils;
/// 
/// #[tokio::test] 
/// async fn test_something() {
///     with_mock_server!(mock_server, {
///         let client = reqwest::Client::new();
///         let user = fetch_github_user(&client, "octocat").await.unwrap();
///         assert_eq!(user.login, "octocat");
///     });
/// }
/// ```
//#[macro_export]
//macro_rules! with_mock_server {
//    ($mock_var:ident, $test_body:block) => {
//        $crate::test_utils::network_mock_test_case::NetworkMockTestCase::register();
//        let $mock_var = $crate::test_utils::mock_server::setup().await;
//        $crate::user::set_base_url_override(Some($mock_var.uri()));
//        $test_body
//        $crate::user::set_base_url_override(None);
//    };
//}

/// Convenience function for simple test scenarios
pub async fn with_mock_server<F, Fut, R>(test_fn: F) -> R 
where
    F: FnOnce(MockServer) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    network_mock_test_case::NetworkMockTestCase::register();
    let mock_server = mock_server::setup().await;

    github_user_fetcher::user::set_base_url_override(Some(mock_server.uri()));

    let result = test_fn(mock_server).await;

    github_user_fetcher::user::set_base_url_override(None);

    result
}

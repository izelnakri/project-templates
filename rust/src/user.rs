//! This module provides functionality to represent and fetch GitHub user information.
//!
//! It defines a `User` struct representing a GitHub user’s public profile fields,
//! along with a thread-safe mechanism to override the GitHub API base URL for testing or alternative endpoints.
//!
//! The main feature is the asynchronous function `fetch_github_user` which fetches user data
//! from GitHub’s REST API using a provided `reqwest::Client`.
//!
//! The module also includes utility methods and tests covering thread safety, printing, and API interaction.

use serde::{Deserialize, Serialize};
use crate::adapter::{HttpAdapter};

/// Represents a GitHub user’s public profile.
#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    /// GitHub login username.
    pub login: String,
    /// User’s display name (optional).
    pub name: Option<String>,
    /// User’s company (optional).
    pub company: Option<String>,
    /// User’s location (optional).
    pub location: Option<String>,
}

impl User {
    /// Prints the user’s information to stdout.
    ///
    /// Optional fields display `"N/A"` if missing.
    #[allow(dead_code)]
    pub fn print(&self) {
        println!("Login: {}", self.login);
        println!("Name: {}", self.name.as_deref().unwrap_or("N/A"));
        println!("Company: {}", self.company.as_deref().unwrap_or("N/A"));
        println!("Location: {}", self.location.as_deref().unwrap_or("N/A"));
    }
}

/// Fetches a GitHub user’s profile information asynchronously.
///
/// # Arguments
///
/// * `client` - A reference to a `reqwest::Client` to perform the HTTP request.
/// * `username` - The GitHub username to fetch.
///
/// # Returns
///
/// Returns a `Result` with a `User` on success, or an error boxed trait object on failure.
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the status is not success, or JSON deserialization fails.
///
/// # Example
///
/// ```no_run
/// use reqwest::Client;
/// use github_user_fetcher::adapter::{HttpAdapter, DEFAULT_API_BASE_URL};
/// use github_user_fetcher::user::fetch_github_user;
///
/// #[tokio::main]
/// async fn main() {
///     let adapter= HttpAdapter::new(DEFAULT_API_BASE_URL);
///     let user = fetch_github_user(&adapter, "octocat").await.unwrap();
///     println!("User login: {}", user.login);
/// }
/// ```
pub async fn fetch_github_user(
    github_api_adapter: &HttpAdapter,
    username: &str,
) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let response = github_api_adapter.get(format!("/users/{}", username)).send().await?;
    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let user = response.error_for_status()?.json::<User>().await?;
    Ok(user)
}

#[cfg(test)]
#[path = "../tests/utils/mod.rs"]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate, matchers::{method, path}};

    #[test]
    fn test_user_print() {
        let user = User {
            login: "octocat".to_string(),
            name: Some("The Octocat".to_string()),
            company: Some("GitHub".to_string()),
            location: Some("San Francisco".to_string()),
        };

        user.print(); // should print without panicking
    }

    #[test]
    fn test_user_print_with_none_fields() {
        let user = User {
            login: "octocat".to_string(),
            name: None,
            company: None,
            location: None,
        };

        user.print(); // prints N/A for all optional fields
    }

    #[tokio::test]
    async fn test_fetch_github_user_error_invalid_user() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/users/notfound"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;
        let adapter = HttpAdapter::new(mock_server.uri());
        let res = fetch_github_user(&adapter, "notfound").await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_fetch_github_user_success_with_mock() {
        let mock_server = test_utils::mock_github_api::setup().await;
        let adapter = HttpAdapter::new(mock_server.uri());
        let user = fetch_github_user(&adapter, "octocat").await.unwrap();

        assert_eq!(user.login, "octocat");
        assert_eq!(user.name.as_deref(), Some("The Octocat"));
        assert_eq!(user.company.as_deref(), Some("GitHub"));
        assert_eq!(user.location.as_deref(), Some("San Francisco"));
    }

    #[tokio::test]
    async fn test_fetch_github_user_failure_status() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/users/erroruser"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;
        let adapter = HttpAdapter::new(mock_server.uri());
        let res = fetch_github_user(&adapter, "erroruser").await;

        assert!(res.is_err());
        let err_str = format!("{}", res.unwrap_err());
        assert!(err_str.contains("Request failed with status"));
    }
}

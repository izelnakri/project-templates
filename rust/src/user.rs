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
use std::sync::RwLock;

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

const DEFAULT_API_BASE: &str = "https://api.github.com";

/// Global override for the base API URL, protected by a read-write lock for thread safety.
static BASE_URL_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);

/// Sets or clears the global base URL override.
///
/// Passing `Some(url)` overrides the GitHub API base URL.
/// Passing `None` clears the override and reverts to the default URL.
///
/// # Examples
///
/// ```
/// use github_user_fetcher::user::set_base_url_override;
///
/// set_base_url_override(Some("http://localhost:8000".to_string()));
/// set_base_url_override(None);
/// ```
pub fn set_base_url_override(url: Option<String>) {
    let mut w = BASE_URL_OVERRIDE.write().unwrap();
    *w = url;
}

/// Returns the current base API URL.
///
/// If an override is set via [`set_base_url_override`], it returns that,
/// otherwise returns the default GitHub API base URL.
fn get_base_url() -> String {
    if let Some(ref url) = *BASE_URL_OVERRIDE.read().unwrap() {
        url.clone()
    } else {
        DEFAULT_API_BASE.to_string()
    }
}

impl User {
    /// Prints the user’s information to stdout.
    ///
    /// Optional fields display `"N/A"` if missing.
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
/// use github_user_fetcher::user::fetch_github_user;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::builder()
///         .user_agent("rust-poem-github-client")
///         .build()
///         .unwrap();
///
///     let user = fetch_github_user(&client, "octocat").await.unwrap();
///     println!("User login: {}", user.login);
/// }
/// ```
pub async fn fetch_github_user(
    client: &reqwest::Client,
    username: &str,
) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let base_url = get_base_url();
    let url = format!("{}/users/{}", base_url.trim_end_matches('/'), username);
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let user = response.error_for_status()?.json::<User>().await?;
    Ok(user)
}

#[cfg(test)]
static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(()); // Add a test mutex for synchronizing tests that modify the base URL

#[cfg(test)]
mod tests {
    mod mock_server {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/utils/mock_server.rs"));
    }

    use super::*;
    use reqwest::Client;
    use tokio::runtime::Runtime;

    #[test]
    fn test_set_and_get_base_url_override() {
        let _lock = TEST_MUTEX.lock().unwrap(); // Acquire mutex for this test to prevent race conditions
        
        set_base_url_override(None);
        assert_eq!(get_base_url(), DEFAULT_API_BASE.to_string());

        let url = "http://localhost:1234".to_string();
        set_base_url_override(Some(url.clone()));
        assert_eq!(get_base_url(), url);

        set_base_url_override(None);
        assert_eq!(get_base_url(), DEFAULT_API_BASE.to_string());
    }

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

    #[test]
    fn test_get_base_url_thread_safety() {
        use std::thread;

        let _lock = TEST_MUTEX.lock().unwrap(); // Acquire mutex for this test to prevent race conditions
        
        set_base_url_override(None);
        assert_eq!(get_base_url(), DEFAULT_API_BASE.to_string());

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || { // Spawn threads to concurrently read and write base URL override
                    if i % 2 == 0 {
                        set_base_url_override(Some(format!("http://localhost:{}", i)));
                    } else {
                        set_base_url_override(None);
                    }
                    get_base_url() // Just get the base URL after setting
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.join().unwrap(); // Join threads and check that no panic occurs
        }
        
        set_base_url_override(None); // Reset to default state after test
    }

    #[test]
    fn test_fetch_github_user_error_invalid_user() {
        let rt = Runtime::new().unwrap();
        let client = Client::builder()
            .user_agent("rust-poem-github-client")
            .build()
            .unwrap();

        let fut = fetch_github_user(&client, "thisuserdoesnotexist1234567890");

        let res = rt.block_on(fut);
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_fetch_github_user_success_with_mock() {
        let _lock = TEST_MUTEX.lock().unwrap(); // Acquire mutex for this test to prevent race conditions
        let mock_server = mock_server::setup().await;
        let original_url = get_base_url(); // Save the original value to restore it later
        
        set_base_url_override(Some(mock_server.uri())); // Set the base URL for this test

        let client = Client::builder()
            .user_agent("rust-poem-github-client")
            .build()
            .unwrap();
        let user = fetch_github_user(&client, "octocat").await.unwrap();

        assert_eq!(user.login, "octocat");
        assert_eq!(user.name.as_deref(), Some("The Octocat"));
        assert_eq!(user.company.as_deref(), Some("GitHub"));
        assert_eq!(user.location.as_deref(), Some("San Francisco"));

        // Restore the original URL setting after the test
        if original_url == DEFAULT_API_BASE {
            set_base_url_override(None);
        } else {
            set_base_url_override(Some(original_url));
        }
    }

    #[tokio::test]
    async fn test_fetch_github_user_failure_status() {
        use wiremock::{Mock, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let _lock = TEST_MUTEX.lock().unwrap(); // Acquire mutex for this test to prevent race conditions
        let mock_server = wiremock::MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/users/notfound"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let original_url = get_base_url(); // Save the original value to restore it later
    
        set_base_url_override(Some(mock_server.uri()));

        let client = Client::builder()
            .user_agent("rust-poem-github-client")
            .build()
            .unwrap();
        let result = fetch_github_user(&client, "notfound").await;

        assert!(result.is_err());
        let err_str = format!("{}", result.unwrap_err());
        assert!(err_str.contains("Request failed with status"));

        // Restore the original URL setting after the test
        if original_url == DEFAULT_API_BASE {
            set_base_url_override(None);
        } else {
            set_base_url_override(Some(original_url));
        }
    }
}

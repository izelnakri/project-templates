use reqwest::Client;
use std::sync::Arc;

/// HTTP adapter with configurable base URL/hostname
/// 
/// This allows easy testing by pointing to wiremock server
/// instead of real external APIs
#[derive(Clone)]
pub struct HttpAdapter {
    client: Arc<Client>,
    pub base_url: String,
}

pub const DEFAULT_API_BASE_URL: &str = "https://api.github.com";

impl HttpAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Arc::new(
            Client::builder()
                .user_agent("github_user_fetcher HttpAdapter")
                .build()
                .expect("Failed to build HTTP client"),
        );
        
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    pub fn get(&self, path: String) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client.get(&url)
    }
}

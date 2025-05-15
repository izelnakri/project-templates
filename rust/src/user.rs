use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

const DEFAULT_API_BASE: &str = "https://api.github.com";

#[derive(Deserialize, Debug)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
}

impl User {
    pub fn print(&self) {
        println!("Login: {}", self.login);
        println!("Name: {}", self.name.as_deref().unwrap_or("N/A"));
        println!("Company: {}", self.company.as_deref().unwrap_or("N/A"));
        println!("Location: {}", self.location.as_deref().unwrap_or("N/A"));
    }
}

pub async fn fetch_github_user(client: &Client, username: &str) -> Result<User, Box<dyn Error + Send + Sync>> {
    let url = format!("{}/users/{}", DEFAULT_API_BASE, username);
    let user = client.get(&url)
        .send()
        .await?
        .error_for_status()? // return error if status is not 2xx
        .json::<User>()
        .await?;

    Ok(user)
}

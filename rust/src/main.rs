mod config;
mod server;
mod user;

use config::{Config, Mode};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_args(std::env::args());

    match config.mode {
        Mode::Server => {
            println!("Starting server on port {}", config.port);
            server::listen(config.port).await?;
        }
        Mode::Cli => {
            let client = Client::builder()
                .user_agent("rust-poem-github-client")
                .build()?;
            let user = user::fetch_github_user(&client, &config.username).await?;
            user.print();
        }
    }

    Ok(())
}

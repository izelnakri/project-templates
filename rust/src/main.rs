mod adapter;
mod config;
mod server;
mod user;

use adapter::{HttpAdapter, DEFAULT_API_BASE_URL};
use config::{Config, Mode};
use server::HttpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_args(std::env::args());

    match config.mode {
        Mode::Server => {
            println!("Starting server on port {}", config.port);

            let github_api_adapter = HttpAdapter::new(DEFAULT_API_BASE_URL);
            HttpServer::new(config.port).run(github_api_adapter).await?;
        }
        Mode::Cli => {
            let github_api_adapter = HttpAdapter::new(DEFAULT_API_BASE_URL);
            let user = user::fetch_github_user(&github_api_adapter, &config.username).await?;
            user.print();
        }
    }

    Ok(())
}

use poem::{
    handler, listener::TcpListener, Route, Server,
    web::{Json, Path},
    error::Error as PoemError,
};
use serde::Serialize;
use crate::user;
use reqwest::Client;
use std::error::Error;

#[derive(Serialize)]
struct UserResponse {
    login: String,
    name: Option<String>,
    company: Option<String>,
    location: Option<String>,
}

#[handler]
async fn user_handler(Path(username): Path<String>) -> Result<Json<UserResponse>, PoemError> {
    let client = Client::builder()
        .user_agent("rust-poem-github-server")
        .build()
        .map_err(|e| PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

    let user = user::fetch_github_user(&client, &username).await
        .map_err(|e| PoemError::from_string(e.to_string(), poem::http::StatusCode::NOT_FOUND))?;

    Ok(Json(UserResponse {
        login: user.login,
        name: user.name,
        company: user.company,
        location: user.location,
    }))
}

pub async fn run_server(port: u16) -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = Route::new().at("/:username", user_handler);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr);

    Server::new(listener).run(app).await?;

    Ok(())
}

use crate::user;
use poem::{
    endpoint::StaticFilesEndpoint,
    listener::TcpListener,
    middleware::Tracing,
    web::{Path, Data},
    get,
    EndpointExt, Route, Server,
    handler
};
use poem_openapi::{
    payload::Json as ApiJson, 
    // param::Path as ApiPath, // NOTE: Currently buggy on ApiPath(param) extraction when param is string
    Object, OpenApi, OpenApiService,
    ApiResponse
};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Object, Serialize, Deserialize)]
struct UserResponse {
    /// GitHub username
    #[oai(validator(max_length = 100))]
    login: String,

    /// User's display name (if available)
    #[oai(validator(max_length = 255))]
    name: Option<String>,

    /// Company information (if available)
    #[oai(validator(max_length = 255))]
    company: Option<String>,

    /// User's location (if available)
    #[oai(validator(max_length = 255))]
    location: Option<String>,
}

#[derive(Object, Serialize, Deserialize)]
struct ErrorResponse {
    /// Error message
    message: String,
}

#[derive(ApiResponse)]
enum GetUserResponse {
    #[oai(status = 200)]
    Ok(ApiJson<UserResponse>),

    #[oai(status = 404)]
    NotFound(ApiJson<ErrorResponse>),
}

#[handler]
async fn get_user(
    client: Data<&Arc<Client>>,
    Path(username): Path<String>,
) -> GetUserResponse {
    match user::fetch_github_user(&client, &username).await {
        Ok(user) => GetUserResponse::Ok(ApiJson(UserResponse {
            login: user.login,
            name: user.name,
            company: user.company,
            location: user.location,
        })),
        Err(e) => GetUserResponse::NotFound(ApiJson(ErrorResponse {
            message: format!("User `{}` not found: {}", username, e),
        })),
    }
}

struct Api {
    client: Arc<Client>,
}

#[OpenApi]
impl Api {
    #[oai(method = "get", path = "/:username")]
    async fn get_user(
        &self,
        Path(username): Path<String>,
    ) -> GetUserResponse {
        match user::fetch_github_user(&self.client, &username).await {
            Ok(user) => GetUserResponse::Ok(ApiJson(UserResponse {
                login: user.login,
                name: user.name,
                company: user.company,
                location: user.location,
            })),
            Err(err) => GetUserResponse::NotFound(ApiJson(ErrorResponse {
                message: format!("User `{}` not found: {}", username, err),
            })),
        }
    }
}

pub async fn listen(port: u16) -> Result<(), std::io::Error> {
    let client = Arc::new(
        Client::builder()
            .user_agent("rust-poem-github-server")
            .build()
            .expect("Failed to build HTTP client"),
    );
    let api_service = OpenApiService::new(
        Api { client: client.clone() },
        "GitHub User API",
        env!("CARGO_PKG_VERSION"),
    )
    .server(format!("http://localhost:{port}/api"))
    .description("A simple API to fetch GitHub user information")
    .summary("GitHub User Information API"); // NOTE: .spec() prints it

    let openapi_ui = api_service.swagger_ui();
    let cargo_docs = StaticFilesEndpoint::new("target/doc/github_user_fetcher")
        .index_file("index.html");
    let app = Route::new()
        .at("/hello", poem::endpoint::make_sync(|_| "Hello, world!"))
        .at("/:username", get(get_user))
        .nest("/api", api_service)
        .nest("/openapi", openapi_ui)
        .nest("/docs", cargo_docs)
        .with(Tracing)
        .data(client.clone());

    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run(app)
        .await
}

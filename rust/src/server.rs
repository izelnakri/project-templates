use crate::user;
use poem::{
    endpoint::StaticFilesEndpoint,
    listener::TcpListener,
    middleware::Tracing,
    web::{Path as WebPath, Data, Redirect},
    get,
    EndpointExt, Route, Server,
    handler,
};
use poem_openapi::{
    payload::Json,
    param::Path,
    Object, OpenApi, OpenApiService,
    ApiResponse,
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

/// Error response payload.
///
/// Returned when the specified user cannot be found or an error occurs.
#[derive(Object, Serialize, Deserialize, Debug)]
struct ErrorResponse {
    /// Error message
    message: String,
}

/// Response wrapper for the `GET /:username` endpoint.
///
/// Includes status-specific typed responses for OpenAPI documentation.
#[derive(ApiResponse, Debug)]
enum GetUserResponse {
    /// Success: GitHub user was found and data is returned.
    #[oai(status = 200)]
    Ok(Json<UserResponse>),

    /// Failure: GitHub user not found or an error occurred.
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

#[handler]
async fn fetch_user(
    client: Data<&Arc<Client>>,
    WebPath(username): WebPath<String>,
) -> GetUserResponse {
    match user::fetch_github_user(&client, &username).await {
        Ok(user) => GetUserResponse::Ok(Json(UserResponse {
            login: user.login,
            name: user.name,
            company: user.company,
            location: user.location,
        })),
        Err(e) => GetUserResponse::NotFound(Json(ErrorResponse {
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
    async fn fetch_user(
        &self,
        Path(username): Path<String>,
    ) -> GetUserResponse {
        println!("username is {}", &username);
        match user::fetch_github_user(&self.client, &username).await {
            Ok(user) => GetUserResponse::Ok(Json(UserResponse {
                login: user.login,
                name: user.name,
                company: user.company,
                location: user.location,
            })),
            Err(err) => GetUserResponse::NotFound(Json(ErrorResponse {
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
    .summary("GitHub User Information API");

    let openapi_ui = api_service.swagger_ui();
    let cargo_docs = StaticFilesEndpoint::new("docs/target");
    let app = Route::new()
        .at("/hello", poem::endpoint::make_sync(|_| "Hello, world!"))
        .at("/:username", get(fetch_user))
        .nest("/api", api_service)
        .nest("/openapi", openapi_ui)
        .at("/docs", poem::endpoint::make_sync(|_| Redirect::temporary("/docs/target/github_user_fetcher/index.html")))
        .nest("/docs/target", cargo_docs)
        .with(Tracing)
        .data(client.clone());

    Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
        .run(app)
        .await
}

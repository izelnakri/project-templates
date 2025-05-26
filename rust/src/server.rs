// NOTE: You do not have to wrap the Client in an Rc or Arc to reuse it, because it already uses an Arc internally.
// TODO: (!) Add examples and description to: param (query param)
// (!) Add examples and description to: param (on post body)
// (!) Turn route handler examples into doctests, add a reqwest wrapper to accomplish this if needed
// https://chatgpt.com/c/682c0f2f-4db4-8010-939e-5568a0018eae
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

use crate::adapter::{HttpAdapter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Response payload for a successful GitHub user fetch.
///
/// Represents a subset of the public GitHub profile information.
#[derive(Object, Serialize, Deserialize, Debug)]
struct UserResponse {
    /// GitHub username
    ///
    /// Maximum length: 100 characters.
    ///
    /// Example: `"octocat"`
    #[oai(validator(max_length = 100))]
    login: String,

    /// User's display name (if available)
    ///
    /// Maximum length: 255 characters.
    ///
    /// Example: `"The Octocat"`
    #[oai(validator(max_length = 255))]
    name: Option<String>,

    /// Company information (if available)
    ///
    /// Maximum length: 255 characters.
    ///
    /// Example: `"GitHub"`
    #[oai(validator(max_length = 255))]
    company: Option<String>,

    /// User's location (if available)
    ///
    /// Maximum length: 255 characters.
    ///
    /// Example: `"San Francisco"`
    #[oai(validator(max_length = 255))]
    location: Option<String>,
}

/// Error response payload.
///
/// Returned when the specified user cannot be found or an error occurs.
#[derive(Object, Serialize, Deserialize, Debug)]
struct ErrorResponse {
    /// Error message
    ///
    /// Example: `"User 'octocat' not found"`
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

/// Synchronous-style Poem handler for directly routing `/username` path.
///
/// Used outside of OpenAPI and primarily for internal/static routing.
///
/// # Example
///
/// ```bash
/// curl http://localhost:3000/octocat
/// ```
#[handler]
async fn fetch_user(
    github_api_adapter: Data<&Arc<HttpAdapter>>,
    WebPath(username): WebPath<String>,
) -> GetUserResponse {
    match user::fetch_github_user(&github_api_adapter, &username).await {
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

/// API service definition implementing the OpenAPI trait.
struct Api {
    /// Shared HTTP client used to fetch GitHub user data.
    github_api_adapter: Arc<HttpAdapter>, // NOTE: Previously it was Arc<HttpAdapter>
}

#[OpenApi]
impl Api {
    /// Get GitHub user details by username.
    ///
    /// Fetches public profile information from the GitHub API.
    ///
    /// # Path Parameters
    ///
    /// - `username`: GitHub username.
    ///
    /// # Returns
    ///
    /// - `200 OK` with user data if the user exists.
    /// - `404 Not Found` if the user does not exist or an error occurs.
    ///
    /// # Example
    ///
    /// ```sh
    /// echo true # CHANGE TO: curl http://localhost:3000/api/octocat
    /// ```
    #[oai(method = "get", path = "/:username")]
    async fn fetch_user(
        &self,
        Path(username): Path<String>,
    ) -> GetUserResponse {
        println!("username is {}", &username);
        match user::fetch_github_user(&self.github_api_adapter, &username).await {
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

pub struct HttpServer {
    port: u16,
    poem_server: Server<poem::listener::TcpListener<String>, std::convert::Infallible>,
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        let server = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")));
        Self { port: port, poem_server: server }
    }

    pub fn run(self, github_api_adapter: HttpAdapter) -> impl Future<Output = std::result::Result<(), std::io::Error>> {
        let adapter = Arc::new(github_api_adapter);
        let api_service = OpenApiService::new(
            Api { github_api_adapter: adapter.clone() },
            "GitHub User API",
            env!("CARGO_PKG_VERSION"),
        )
        .server(format!("http://localhost:{0}/api", self.port))
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
            .data(adapter.clone());

        self.poem_server.run(app)
    }
}

#[cfg(test)]
#[path = "../tests/utils/mod.rs"]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::{DEFAULT_API_BASE_URL};

    async fn spawn_server(port: u16) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let github_api_adapter = HttpAdapter::new(DEFAULT_API_BASE_URL);
            let _ = HttpServer::new(port).run(github_api_adapter).await;
        })
    }

    #[tokio::test]
    async fn test_listen_success_on_ports() {
        let port1 = 3100;
        let port2 = 3101;

        let handle1 = spawn_server(port1).await;
        let handle2 = spawn_server(port2).await;

        assert!(test_utils::wait_for_port_open(port1).await, "Port {port1} should open quickly");
        assert!(test_utils::wait_for_port_open(port2).await, "Port {port2} should open quickly");

        handle1.abort();
        handle2.abort();
    }

    #[tokio::test]
    async fn test_listen_fails_on_duplicate_port() {
        let port = 3200;

        let handle = spawn_server(port).await;
        assert!(test_utils::wait_for_port_open(port).await, "Port {port} should be open");

        let github_api_adapter = HttpAdapter::new(DEFAULT_API_BASE_URL);
        let result = HttpServer::new(port).run(github_api_adapter).await;
        assert!(result.is_err(), "Expected listen to fail on duplicate port");

        handle.abort();
    }

    #[tokio::test] 
    async fn test_fetch_user_handler_success() {
        let mock_server = test_utils::mock_github_api::setup().await;
        let adapter = Arc::new(HttpAdapter::new(mock_server.uri())); // NOTE: should have an MockServer error?
        let api = Api { github_api_adapter: adapter.clone() }; // NOTE: Make it an adapter with base_url
        let resp = api.fetch_user(Path("octocat".to_string())).await;
        if let GetUserResponse::Ok(Json(user_response)) = resp {
            assert_eq!(user_response.login, "octocat");
            assert_eq!(user_response.name.as_deref(), Some("The Octocat"));
            assert_eq!(user_response.company.as_deref(), Some("GitHub"));
            assert_eq!(user_response.location.as_deref(), Some("San Francisco"));
        } else {
            panic!("Expected GetUserResponse::Ok but got smt else")
        }
    }
}

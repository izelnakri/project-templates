//! # GitHub User API Server
//!
//! This module provides a REST API server for fetching and managing GitHub user information.
//! The server is built using the Poem web framework and provides OpenAPI documentation.
//!
//! ## Features
//!
//! - Fetch individual GitHub users by username
//! - Create users with auto-incrementing IDs
//! - Search users with pagination
//! - List users with pagination
//! - Built-in statistics endpoint
//! - OpenAPI/Swagger documentation
//! - Static file serving for documentation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use github_user_fetcher::adapter::HttpAdapter;
//! use github_user_fetcher::server::HttpServer;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let adapter = HttpAdapter::new("https://api.github.com");
//! let server = HttpServer::new(3000);
//! server.run(adapter).await.unwrap();
//! # Ok(())
//! # }
//! ```

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
    param::{Path, Query as OpenApiQuery},
    Object, OpenApi, OpenApiService,
    ApiResponse,
};

mod stats;

use crate::adapter::{HttpAdapter};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

/// Global counter for generating unique user IDs - starts from 1
static USER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// GitHub user's public profile information
#[derive(Object, Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    /// GitHub username. Example: `"octocat"`
    #[oai(validator(max_length = 100))]
    pub login: String,

    /// User's display name. Example: `"The Octocat"`
    #[oai(validator(max_length = 255))]
    pub name: Option<String>,

    /// User's company. Example: `"GitHub"`
    #[oai(validator(max_length = 255))]
    pub company: Option<String>,

    /// User's location. Example: `"San Francisco, CA"`
    #[oai(validator(max_length = 255))]
    pub location: Option<String>,
}

impl UserResponse {
    /// Adds an ID to create a UserWithIdResponse
    /// 
    /// ```rust
    /// use github_user_fetcher::server::{UserResponse};
    ///
    /// let user = UserResponse { 
    ///     login: "octocat".to_string(), 
    ///     name: Some("The Octocat".to_string()),
    ///     company: Some("GitHub".to_string()),
    ///     location: Some("San Francisco".to_string())
    /// };
    /// let user_with_id = user.with_id(42);
    /// assert_eq!(user_with_id.id, 42);
    /// ```
    pub fn with_id(self, id: u64) -> UserWithIdResponse {
        UserWithIdResponse {
            id,
            user: self,
        }
    }
}

/// User response with system-generated ID
#[derive(Object, Serialize, Deserialize, Debug)]
pub struct UserWithIdResponse {
    /// Auto-incremented ID starting from 1. Example: `42`
    pub id: u64,

    /// User data flattened into parent object
    #[serde(flatten)]
    #[oai(flatten)]
    user: UserResponse,
}

/// Request to create a new user
#[derive(Object, Deserialize, Debug)]
struct CreateUserRequest {
    /// GitHub username to fetch. Example: `"octocat"`
    username: String,
}

/// Paginated search results for users
#[derive(Object, Serialize, Deserialize, Debug)]
struct SearchUsersResponse {
    /// Total matching users. Example: `1234`
    total_count: u64,

    /// Current page of user results
    items: Vec<UserResponse>,
}

/// Paginated user list with cursor
#[derive(Object, Serialize, Deserialize, Debug)]
struct ListUsersResponse {
    /// Users in current page
    users: Vec<UserResponse>,

    /// Cursor for next page. Example: `583231`
    since: Option<u64>,
}

/// Standard error response
#[derive(Object, Serialize, Deserialize, Debug)]
struct ErrorResponse {
    /// Error description. Example: `"User 'nonexistent' not found: 404 Not Found"`
    message: String,
}

/// Response for get user endpoint
#[derive(ApiResponse, Debug)]
enum GetUserResponse {
    /// User found - returns user data
    #[oai(status = 200)]
    Ok(Json<UserResponse>),

    /// User not found or API error
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

/// Response for create user endpoint
#[derive(ApiResponse, Debug)]
enum CreateUserResponse {
    /// User created with auto-generated ID
    #[oai(status = 201)]
    Created(Json<UserWithIdResponse>),

    /// User not found on GitHub
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

/// Response for search users endpoint
#[derive(ApiResponse, Debug)]
enum SearchUsersApiResponse {
    /// Search completed successfully
    #[oai(status = 200)]
    Ok(Json<SearchUsersResponse>),

    /// Invalid parameters or API error
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
}

/// Response for list users endpoint
#[derive(ApiResponse, Debug)]
enum ListUsersApiResponse {
    /// User list retrieved successfully
    #[oai(status = 200)]
    Ok(Json<ListUsersResponse>),

    /// Invalid parameters or API error
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
}

/// Legacy handler for non-OpenAPI user fetch route
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

/// Main API implementation with all endpoint handlers
struct Api {
    /// HTTP adapter for GitHub API requests
    github_api_adapter: Arc<HttpAdapter>,
}

#[OpenApi]
impl Api {
    /// Fetch a GitHub user by username
    #[oai(method = "get", path = "/:username")]
    async fn fetch_user(
        &self,
        /// The GitHub username to fetch profile for (e.g., "octocat", "torvalds")
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

    /// Create a new user with auto-generated ID
    #[oai(method = "post", path = "/users")]
    async fn create_user(
        &self,
        Json(request): Json<CreateUserRequest>,
    ) -> CreateUserResponse {
        match user::fetch_github_user(&self.github_api_adapter, &request.username).await {
            Ok(user) => {
                let id = USER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                let user_response = UserResponse {
                    login: user.login,
                    name: user.name,
                    company: user.company,
                    location: user.location,
                };
                CreateUserResponse::Created(Json(user_response.with_id(id)))
            }
            Err(err) => CreateUserResponse::NotFound(Json(ErrorResponse {
                message: format!("User `{}` not found: {}", request.username, err),
            })),
        }
    }

    /// Search GitHub users with pagination
    #[oai(method = "get", path = "/users/search")]
    async fn search_users(
        &self,
        /// Search query string to match against usernames (e.g., "octo", "john")
        OpenApiQuery(query): OpenApiQuery<String>,
        /// Number of results per page, maximum 100 (default: 30)
        OpenApiQuery(per_page): OpenApiQuery<Option<u32>>,
        /// Page number for pagination, starts at 1 (default: 1)
        OpenApiQuery(page): OpenApiQuery<Option<u32>>,
    ) -> SearchUsersApiResponse {
        let response = match self.github_api_adapter
            .get("/search/users".to_string())
            .query(&[
                ("q", &query),
                ("type", &"user".to_string()), // TODO: Can this be string?
                ("per_page", &per_page.unwrap_or(30).min(100).to_string()),
                ("page", &page.unwrap_or(1).to_string()),
            ])
            .send()
            .await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => text,
                    Err(e) => return SearchUsersApiResponse::BadRequest(Json(ErrorResponse {
                        message: format!("Failed to read response: {}", e),
                    })),
                },
                Err(e) => return SearchUsersApiResponse::BadRequest(Json(ErrorResponse {
                    message: format!("Search failed: {}", e),
                })),
            };
        let json: serde_json::Value = match serde_json::from_str(&response) {
            Ok(j) => j,
            Err(e) => return SearchUsersApiResponse::BadRequest(Json(ErrorResponse {
                message: format!("Failed to parse search response: {}", e),
            })),
        };
        let total_count = json["total_count"].as_u64().unwrap_or(0);
        let items: Vec<UserResponse> = json["items"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();

        SearchUsersApiResponse::Ok(Json(SearchUsersResponse { total_count, items }))
    }

    /// List GitHub users with cursor-based pagination
    #[oai(method = "get", path = "/users")]
    async fn list_users(
        &self,
        /// Cursor for pagination, use the user ID from where to start listing (optional)
        OpenApiQuery(since): OpenApiQuery<Option<u64>>,
        /// Number of users per page, maximum 100 (default: 30)
        OpenApiQuery(per_page): OpenApiQuery<Option<u32>>,
    ) -> ListUsersApiResponse {
        let per_page = per_page.unwrap_or(30).min(100);
        let list_url = match since {
            Some(since_id) => format!("/users?per_page={}&since={}", per_page, since_id),
            None => format!("/users?per_page={}", per_page),
        };
        let response = match self.github_api_adapter.get(list_url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(e) => return ListUsersApiResponse::BadRequest(Json(ErrorResponse {
                    message: format!("Failed to read response: {}", e),
                })),
            },
            Err(e) => return ListUsersApiResponse::BadRequest(Json(ErrorResponse {
                message: format!("Failed to list users: {}", e),
            })),
        };
        let users_json: Vec<serde_json::Value> = match serde_json::from_str(&response) {
            Ok(users) => users,
            Err(e) => return ListUsersApiResponse::BadRequest(Json(ErrorResponse {
                message: format!("Failed to parse users response: {}", e),
            })),
        };
        let users: Vec<UserResponse> = users_json
            .iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        let next_since = users_json.last().and_then(|user| user["id"].as_u64());

        ListUsersApiResponse::Ok(Json(ListUsersResponse {
            users,
            since: next_since,
        }))
    }

    /// Get API statistics and health information
    #[oai(method = "get", path = "/stats")]
    async fn get_stats(&self) -> Json<stats::StatsResponse> {
        Json(stats::get_stats())
    }
}

/// HTTP server for the GitHub User API
pub struct HttpServer {
    /// Port number to bind to. Example: `3000`
    pub port: u16,
    
    /// Pre-configured Poem server instance
    poem_server: Server<poem::listener::TcpListener<String>, std::convert::Infallible>,
}

impl HttpServer {
    /// Create a new HTTP server instance
    /// 
    /// ```rust
    /// use github_user_fetcher::server::{HttpServer};
    ///
    /// let server = HttpServer::new(3000);
    /// assert_eq!(server.port, 3000);
    /// ```
    pub fn new(port: u16) -> Self {
        let server = Server::new(TcpListener::bind(format!("0.0.0.0:{port}")));
        Self { port: port, poem_server: server }
    }

    /// Start the HTTP server with GitHub API adapter
    pub fn run(self, github_api_adapter: HttpAdapter) -> impl std::future::Future<Output = std::result::Result<(), std::io::Error>> {
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

/// Test utilities for mock servers and helpers
#[cfg(test)]
#[path = "../../tests/utils/mod.rs"]
mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::{DEFAULT_API_BASE_URL};

    /// Spawn a test server on specified port
    async fn spawn_server(port: u16) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let github_api_adapter = HttpAdapter::new(DEFAULT_API_BASE_URL);
            let _ = HttpServer::new(port).run(github_api_adapter).await;
        })
    }

    /// Test server binding to different ports
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

    /// Test server fails on duplicate port binding
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

    /// Test successful user fetching with mock API
    #[tokio::test] 
    async fn test_fetch_user_handler_success() {
        let mock_server = test_utils::mock_github_api::setup().await;
        let adapter = Arc::new(HttpAdapter::new(mock_server.uri()));
        let api = Api { github_api_adapter: adapter.clone() };
        let response = api.fetch_user(Path("octocat".to_string())).await;
        if let GetUserResponse::Ok(Json(user)) = response {
            assert_eq!(user.login, "octocat");
            assert_eq!(user.name.as_deref(), Some("The Octocat"));
            assert_eq!(user.company.as_deref(), Some("GitHub"));
            assert_eq!(user.location.as_deref(), Some("San Francisco"));
        } else {
            panic!("Expected GetUserResponse::Ok but got something else")
        }
    }

    /// Test user creation with auto-incrementing IDs
    #[tokio::test]
    async fn test_create_user_with_incremental_id() {
        let mock_server = test_utils::mock_github_api::setup().await;
        let adapter = Arc::new(HttpAdapter::new(mock_server.uri()));
        let api = Api { github_api_adapter: adapter.clone() };
        
        USER_ID_COUNTER.store(1, Ordering::SeqCst);  // Reset counter for test consistency
        
        let request = CreateUserRequest {
            username: "octocat".to_string(),
        };
        let response = api.create_user(Json(request)).await;
        
        if let CreateUserResponse::Created(Json(user)) = response {
            assert_eq!(user.id, 1);
            assert_eq!(user.user.login, "octocat");
        } else {
            panic!("Expected CreateUserResponse::Created but got something else")
        }
        
        let request2 = CreateUserRequest {
            username: "octocat".to_string(),
        };
        let response2 = api.create_user(Json(request2)).await;
        
        if let CreateUserResponse::Created(Json(user2)) = response2 {
            assert_eq!(user2.id, 2);
        } else {
            panic!("Expected CreateUserResponse::Created but got something else")
        }
    }
}

// TODO: Missing tests for GET /users/search and GET /users

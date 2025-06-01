// NOTE: You do not have to wrap the Client in an Rc or Arc to reuse it, because it already uses an Arc internally.
// TODO: Simplification, Add missing routes, docs & examples
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
    param::{Path, Query as OpenApiQuery},
    Object, OpenApi, OpenApiService,
    ApiResponse,
};

mod stats;

use crate::adapter::{HttpAdapter};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

// Global counter for user IDs (in-memory, not persistent)
static USER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

// TODO: This can probably be simplified
// Helper function to parse user from JSON value
fn parse_user_response(item: &serde_json::Value) -> Option<UserResponse> {
    let login = item["login"].as_str()?.to_string();
    let name = item["name"].as_str().map(String::from);
    let company = item["company"].as_str().map(String::from);
    let location = item["location"].as_str().map(String::from);
    
    Some(UserResponse { login, name, company, location })
}

/// Response payload for a successful GitHub user fetch.
///
/// Represents a subset of the public GitHub profile information.
#[derive(Object, Serialize, Deserialize, Debug, Clone)]
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

impl UserResponse {
    /// Convert UserResponse to UserWithIdResponse by adding an ID
    fn with_id(self, id: u64) -> UserWithIdResponse {
        UserWithIdResponse {
            id,
            user: self,
        }
    }
}

/// Response payload with ID for POST requests
///
/// Extends UserResponse with an incremental ID field
#[derive(Object, Serialize, Deserialize, Debug)]
struct UserWithIdResponse {
    /// Incremental ID (in-memory only)
    ///
    /// Example: `1`
    id: u64,

    // TODO: Do you need both?
    /// User information
    #[serde(flatten)]
    #[oai(flatten)]
    user: UserResponse,
}

/// Request payload for creating a user
#[derive(Object, Deserialize, Debug)]
struct CreateUserRequest {
    /// GitHub username to fetch
    ///
    /// Example: `"octocat"`
    username: String,
}

/// Search users response
#[derive(Object, Serialize, Deserialize, Debug)]
struct SearchUsersResponse {
    /// Total number of users found
    total_count: u64,

    /// Array of users matching the search query
    items: Vec<UserResponse>,
}

/// List users response
#[derive(Object, Serialize, Deserialize, Debug)]
struct ListUsersResponse {
    /// Array of users
    users: Vec<UserResponse>,

    /// Next user ID for pagination (if available)
    since: Option<u64>,
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

/// Response wrapper for the `POST /users` endpoint.
#[derive(ApiResponse, Debug)]
enum CreateUserResponse {
    /// Success: GitHub user was found and created with ID.
    #[oai(status = 201)]
    Created(Json<UserWithIdResponse>),

    /// Failure: GitHub user not found or an error occurred.
    #[oai(status = 404)]
    NotFound(Json<ErrorResponse>),
}

/// Response wrapper for search users endpoint.
#[derive(ApiResponse, Debug)]
enum SearchUsersApiResponse {
    /// Success: Search completed successfully.
    #[oai(status = 200)]
    Ok(Json<SearchUsersResponse>),

    /// Failure: Search failed or error occurred.
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
}

/// Response wrapper for list users endpoint.
#[derive(ApiResponse, Debug)]
enum ListUsersApiResponse {
    /// Success: Users listed successfully.
    #[oai(status = 200)]
    Ok(Json<ListUsersResponse>),

    /// Failure: List failed or error occurred.
    #[oai(status = 400)]
    BadRequest(Json<ErrorResponse>),
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
    github_api_adapter: Arc<HttpAdapter>,
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
    /// curl http://localhost:3000/api/octocat
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

    /// Create a user by fetching from GitHub and assigning an incremental ID.
    ///
    /// Fetches user data from GitHub API and assigns an in-memory incremental ID.
    /// The ID counter is not persistent and resets when the server restarts.
    ///
    /// # Request Body
    ///
    /// - `username`: GitHub username to fetch and create.
    ///
    /// # Returns
    ///
    /// - `201 Created` with user data including incremental ID if successful.
    /// - `404 Not Found` if the user does not exist or an error occurs.
    ///
    /// # Example
    ///
    /// ```sh
    /// curl -X POST http://localhost:3000/api/users \
    ///   -H "Content-Type: application/json" \
    ///   -d '{"username": "octocat"}'
    /// ```
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

    /// Search GitHub users by query.
    ///
    /// Searches for GitHub users using the GitHub Search API.
    ///
    /// # Query Parameters
    ///
    /// - `query`: Search query string (required).
    /// - `per_page`: Number of results per page (optional, default: 30, max: 100).
    /// - `page`: Page number (optional, default: 1).
    ///
    /// # Returns
    ///
    /// - `200 OK` with search results.
    /// - `400 Bad Request` if the query is invalid or an error occurs.
    ///
    /// # Example
    ///
    /// ```sh
    /// curl "http://localhost:3000/api/users/search?query=octocat&per_page=10&page=1"
    /// ```
    #[oai(method = "get", path = "/users/search")]
    async fn search_users(
        &self,
        OpenApiQuery(query): OpenApiQuery<String>,
        OpenApiQuery(per_page): OpenApiQuery<Option<u32>>,
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
            .filter_map(parse_user_response)
            .collect();

        SearchUsersApiResponse::Ok(Json(SearchUsersResponse { total_count, items }))
    }

    /// List GitHub users with pagination.
    ///
    /// Lists GitHub users using the GitHub Users API with cursor-based pagination.
    ///
    /// # Query Parameters
    ///
    /// - `since`: User ID to start listing from (optional).
    /// - `per_page`: Number of results per page (optional, default: 30, max: 100).
    ///
    /// # Returns
    ///
    /// - `200 OK` with list of users and pagination info.
    /// - `400 Bad Request` if parameters are invalid or an error occurs.
    ///
    /// # Example
    ///
    /// ```sh
    /// curl "http://localhost:3000/api/users?since=1000&per_page=10"
    /// ```
    #[oai(method = "get", path = "/users")]
    async fn list_users(
        &self,
        OpenApiQuery(since): OpenApiQuery<Option<u64>>,
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
            .filter_map(parse_user_response)
            .collect();
        let next_since = users_json.last().and_then(|user| user["id"].as_u64());

        ListUsersApiResponse::Ok(Json(ListUsersResponse {
            users,
            since: next_since,
        }))
    }

    #[oai(method = "get", path = "/stats")]
    async fn get_stats(&self) -> Json<stats::StatsResponse> {
        Json(stats::get_stats())
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

#[cfg(test)]
#[path = "../../tests/utils/mod.rs"]
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

    #[tokio::test]
    async fn test_create_user_with_incremental_id() {
        let mock_server = test_utils::mock_github_api::setup().await;
        let adapter = Arc::new(HttpAdapter::new(mock_server.uri()));
        let api = Api { github_api_adapter: adapter.clone() };
        
        // Reset counter for test consistency
        USER_ID_COUNTER.store(1, Ordering::SeqCst);
        
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

    // TODO: Missing tests for GET /users/search and GET /users
}

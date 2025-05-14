/**
 * @file server.hpp
 * @brief Declares the interface for starting a simple HTTP server.
 *
 * This server listens on a given port and handles HTTP GET requests to fetch GitHub user information.
 * When a client sends a GET request to `/username`, the server responds with a JSON object containing
 * user details (login, name, company, and location) retrieved from the GitHub API.
 *
 * Example:
 * @code
 * start_http_server(8080); // Starts the server on port 8080
 * // Then access: http://localhost:8080/octocat
 * @endcode
 *
 * The server supports graceful shutdown via SIGINT and SIGTERM.
 */

#ifndef SERVER_HPP
#define SERVER_HPP

/**
 * @brief Starts an HTTP server that listens on the specified port.
 *
 * The server handles GET requests in the format `/username`, where `username` is a GitHub login.
 * For each request:
 * - If the user is found, responds with HTTP 200 and a JSON body.
 * - If the user is not found or an error occurs, responds with HTTP 404 and an error message.
 * - If the request URI is malformed or missing a username, responds with HTTP 400.
 *
 * @param port The TCP port on which the server should listen for incoming connections.
 */
void start_http_server(int port);

#endif // SERVER_HPP

#ifndef SERVER_HPP
#define SERVER_HPP

/**
 * @brief Starts an HTTP server that listens on the specified port.
 * 
 * The server responds to GET /username requests by returning JSON user info.
 *
 * @param port The TCP port to listen on.
 */
void start_http_server(int port);

#endif // SERVER_HPP

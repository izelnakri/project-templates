/**
 * @file server.h
 * @brief Declaration of the HTTP server functions.
 */

#ifndef SERVER_H
#define SERVER_H

/**
 * @brief Starts the HTTP server.
 *
 * Initializes and runs the Civetweb server on the specified port.
 *
 * @param port TCP port number to listen on.
 */
void start_http_server(int port);

#endif

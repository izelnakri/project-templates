/**
 * @file server.c
 * @brief Implements a simple HTTP server using Civetweb to serve GitHub user
 * information.
 */

#include "server.h"
#include "../vendor/civetweb/civetweb.h"
#include "user.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/**
 * @brief HTTP request handler for user lookup.
 *
 * Parses the request URI to extract the username, fetches the corresponding
 * GitHub user, and responds with a JSON object.
 *
 * @param conn The connection to the client.
 * @param cbdata Callback data (unused).
 * @return HTTP status code (e.g., 200 for success, 400 for bad request).
 */
static int user_handler(struct mg_connection *conn, void *cbdata) {
  (void)cbdata; // Mark parameter as deliberately unused
  const struct mg_request_info *req_info = mg_get_request_info(conn);
  const char *uri = req_info->local_uri;

  // Validate that the URI is valid and null-terminated
  size_t uri_len = strnlen(uri, 1024); // Limit max length to avoid over-read
  if (uri_len < 2) {
    mg_printf(conn, "HTTP/1.1 400 Bad Request\r\nContent-Type: "
                    "text/plain\r\n\r\nMissing username\n");
    return 400;
  }

  const char *username = uri + 1; // skip the initial '/'
  User user = fetch_github_user(username);

  mg_printf(conn, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n");
  mg_printf(conn,
            "{\n  \"login\": \"%s\",\n  \"name\": \"%s\",\n  \"company\": "
            "\"%s\",\n  \"location\": \"%s\"\n}\n",
            user.login, user.name, user.company, user.location);

  return 200;
}

/**
 * @brief Starts an HTTP server on the specified port.
 *
 * Configures Civetweb with basic options and handles incoming user requests.
 * The server runs indefinitely until manually interrupted.
 *
 * @param port The TCP port to listen on.
 */
void start_http_server(int port) {
  char *port_str = NULL;
  if (asprintf(&port_str, "%d", port) == -1) {
    (void)fprintf(stderr, "Failed to allocate memory for port\n");
    return;
  }

  const char *options[] = {"listening_ports",
                           port_str,
                           "enable_keep_alive",
                           "yes",
                           "access_log_file",
                           "access.log",
                           NULL};

  struct mg_callbacks callbacks = {0};
  struct mg_context *ctx = mg_start(&callbacks, NULL, options);

  if (!ctx) {
    (void)fprintf(stderr, "Failed to start Civetweb server\n");
    return;
  }

  mg_set_request_handler(ctx, "/", user_handler, NULL);

  printf("Server started on port %s. Visit http://localhost:%s/USERNAME\n",
         port_str, port_str);
  puts("Press Ctrl+C to quit.");
  free(port_str);

  // Keep running (or use signal handler in future)
  while (1) {
    sleep(1);
  }

  mg_stop(ctx);
}

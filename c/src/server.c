#include "user.h"
#include "server.h"
#include "vendor/civetweb/civetweb.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int user_handler(struct mg_connection *conn, void *cbdata) {
    const struct mg_request_info *req_info = mg_get_request_info(conn);
    const char *uri = req_info->local_uri;

    if (!uri || strlen(uri) < 2) {
        mg_printf(conn,
                  "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nMissing username\n");
        return 400;
    }

    const char *username = uri + 1; // skip the initial '/'
    User user = fetch_github_user(username);

    mg_printf(conn,
              "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n");
    mg_printf(conn,
              "{\n  \"login\": \"%s\",\n  \"name\": \"%s\",\n  \"company\": \"%s\",\n  \"location\": \"%s\"\n}\n",
              user.login, user.name, user.company, user.location);

    return 200;
}

void start_http_server(int port) {
    char port_str[16];
    snprintf(port_str, sizeof(port_str), "%d", port);

    const char *options[] = {
        "listening_ports", port_str,
        "enable_keep_alive", "yes",
        "access_log_file", "access.log",
        NULL
    };

    struct mg_callbacks callbacks = {0};
    struct mg_context *ctx = mg_start(&callbacks, NULL, options);

    if (!ctx) {
        fprintf(stderr, "Failed to start Civetweb server\n");
        return;
    }

    mg_set_request_handler(ctx, "/", user_handler, NULL);

    printf("Server started on port %s. Visit http://localhost:%s/USERNAME\n", port_str, port_str);
    puts("Press Ctrl+C to quit.");

    // Keep running (or use signal handler in future)
    while (1) {
        sleep(1);
    }

    mg_stop(ctx);
}

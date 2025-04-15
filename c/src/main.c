#include "user.h"
#include "server.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    const char *username = "izelnakri";
    int port = 1234;
    int run_server = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--user") == 0 && i + 1 < argc) {
            username = argv[i + 1];
            i++;
        } else if (strncmp(argv[i], "--user=", 7) == 0) {
            username = argv[i] + 7;
        } else if (strcmp(argv[i], "--server") == 0) {
            run_server = 1;
        } else if (strncmp(argv[i], "--port=", 7) == 0) {
            port = atoi(argv[i] + 7);
        }
    }

    if (run_server) {
        start_http_server(port);
    } else {
        User user = fetch_github_user(username);
        print_github_user(&user);
    }

    return 0;
}

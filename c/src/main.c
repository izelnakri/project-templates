/**
 * @file main.c
 * @brief Entry point for the application. Parses command-line arguments
 *        and either starts the HTTP server or fetches and displays a GitHub
 * user.
 */

#include "server.h"
#include "user.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * @brief Main function that initializes and runs the application.
 *
 * Supported command-line arguments:
 * - `--user <username>` : Specify the GitHub username to fetch.
 * - `--user=<username>` : Alternative way to specify the GitHub username.
 * - `--server`          : Run the HTTP server instead of fetching a user.
 * - `--port=<port>`     : Specify the port for the HTTP server (default: 1234).
 *
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return int Exit status code (0 for success, nonzero for failure).
 */
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
      char *endptr = NULL;
      port = (int)strtol(argv[i] + 7, &endptr, 10);
      if (endptr == argv[i] + 7 || *endptr != '\0') {
        (void)fprintf(stderr, "Invalid port number: %s\n", argv[i] + 7);
        exit(EXIT_FAILURE);
      }
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

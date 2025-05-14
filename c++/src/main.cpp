/**
 * @file main.cpp
 * @brief Entry point for the application. Parses command-line arguments
 *        and either starts the HTTP server or fetches and displays a GitHub
 * user.
 *
 * This file contains the main function which acts as the orchestrator of the
 * program. It interprets the configuration from the user, and based on that
 * configuration, it either launches a local HTTP server or performs a GitHub
 * API request to fetch user data, then displays it to stdout.
 */

#include "config.hpp"
#include "server.hpp"
#include "user.hpp"

#include <iostream>

/**
 * @brief Main function that initializes and runs the application.
 *
 * This function performs the following steps:
 * - Parses command-line arguments into an AppConfig object.
 * - Based on the parsed configuration:
 *   - If `run_server` is true, launches the HTTP server on the specified port.
 *   - Otherwise, fetches a GitHub user and prints the result.
 *
 * Any exceptions thrown during the execution are caught and logged to `stderr`,
 * with a non-zero return code indicating failure.
 *
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return int Exit status code (0 for success, non-zero for failure).
 */
int main(int argc, char **argv) {
  try {
    // Convert argv to vector<string> for easier handling
    std::vector<std::string> args(argv + 1, argv + argc);

    // Parse arguments into configuration
    AppConfig config = parse_arguments(args);

    if (config.run_server) {
      // Start HTTP server if --server option is set
      start_http_server(config.port);
    } else {
      // Otherwise fetch and display GitHub user info
      User user = User::fetch_github_user(config.username);
      user.print();
    }

    return 0;
  } catch (const std::exception &ex) {
    std::cerr << "Error: " << ex.what() << "\n";
    return 1;
  }
}

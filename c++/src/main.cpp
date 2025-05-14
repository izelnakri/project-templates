/**
 * @file main.cpp
 * @brief Entry point for the application. Parses command-line arguments
 *        and either starts the HTTP server or fetches and displays a GitHub
 * user.
 */

#include "config.hpp"
#include "server.hpp"
#include "user.hpp"

#include <iostream>

/**
 * @brief Main function that initializes and runs the application.
 *
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return int Exit status code (0 for success, nonzero for failure).
 */
int main(int argc, char **argv) {
  try {
    std::vector<std::string> args(
        argv + 1,
        argv + argc); // Convert argv to vector<string> for easier handling

    AppConfig config = parse_arguments(args);

    if (config.run_server) {
      start_http_server(config.port);
    } else {
      User user = User::fetch_github_user(config.username);
      user.print();
    }

    return 0;
  } catch (const std::exception &ex) {
    std::cerr << "Error: " << ex.what() << "\n";
    return 1;
  }
}

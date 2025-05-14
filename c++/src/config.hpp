/**
 * @file config.hpp
 * @brief Declares configuration structures and functions for argument parsing.
 *
 * This header defines the `AppConfig` structure used to store settings parsed
 * from the command line, and declares the `parse_arguments` function which
 * fills this structure based on user input.
 */

#pragma once

#include <string>
#include <vector>

/**
 * @brief Struct holding parsed command-line configuration.
 *
 * The `AppConfig` structure stores options provided by the user at runtime:
 * - `username`: The GitHub username to fetch (default: "izelnakri").
 * - `port`: The port on which the HTTP server should run (default: 1234).
 * - `run_server`: Whether the application should start the HTTP server.
 */
struct AppConfig {
  std::string username = "izelnakri";  ///< GitHub username to fetch
  int port = 1234;                     ///< Port for the HTTP server
  bool run_server = false;             ///< Flag to run server mode
};

/**
 * @brief Parses command-line arguments into AppConfig.
 *
 * Accepts a vector of command-line arguments and fills an `AppConfig` struct
 * with the extracted values. Supported flags:
 * - `--user <username>` or `--user=<username>`: Set the GitHub username.
 * - `--server`: Enable server mode.
 * - `--port=<number>`: Set the port number.
 *
 * @param args Vector of arguments (excluding program name).
 * @return Parsed AppConfig.
 * @throws std::invalid_argument If the port value is invalid (non-numeric).
 */
AppConfig parse_arguments(const std::vector<std::string> &args);

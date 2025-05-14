/**
 * @file config.cpp
 * @brief Contains logic for parsing command-line arguments into an AppConfig
 * object.
 *
 * This file provides the implementation of a utility function that translates
 * raw command-line input into structured configuration for the application.
 */

#include "config.hpp"

#include <stdexcept>

/**
 * @brief Parses command-line arguments into an AppConfig structure.
 *
 * This function iterates over the provided command-line arguments and looks
 * for:
 * - `--user <username>` or `--user=<username>`: Sets the GitHub username to
 * fetch.
 * - `--server`: Indicates that the HTTP server should be started.
 * - `--port=<number>`: Sets the port number for the HTTP server.
 *
 * Invalid port values will cause an exception to be thrown.
 *
 * @param args A list of command-line arguments, excluding the program name.
 * @return AppConfig A structure populated with the extracted configuration.
 *
 * @throws std::invalid_argument If the port value is not a valid integer.
 */
AppConfig parse_arguments(const std::vector<std::string> &args) {
  AppConfig config;

  for (size_t i = 0; i < args.size(); ++i) {
    const std::string &arg = args[i];

    if (arg == "--user" && i + 1 < args.size()) {
      config.username = args[++i];
    } else if (arg.starts_with("--user=")) {
      config.username = arg.substr(7);
    } else if (arg == "--server") {
      config.run_server = true;
    } else if (arg.starts_with("--port=")) {
      try {
        config.port = std::stoi(arg.substr(7));
      } catch (const std::exception &) {
        throw std::invalid_argument("Invalid port number: " + arg.substr(7));
      }
    }
  }

  return config;
}

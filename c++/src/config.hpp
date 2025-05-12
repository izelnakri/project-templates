#pragma once

#include <string>
#include <vector>

/**
 * @brief Struct holding parsed command-line configuration.
 */
struct AppConfig {
    std::string username = "izelnakri";
    int port = 1234;
    bool run_server = false;
};

/**
 * @brief Parses command-line arguments into AppConfig.
 *
 * @param args Vector of arguments (excluding program name).
 * @return Parsed AppConfig.
 * @throws std::invalid_argument if port parsing fails.
 */
AppConfig parse_arguments(const std::vector<std::string>& args);

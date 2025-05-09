/**
 * @file main.cpp
 * @brief Entry point for the application. Parses command-line arguments
 *        and either starts the HTTP server or fetches and displays a GitHub
 * user.
 */

#include "server.hpp"
#include "user.hpp"

#include <iostream>
#include <string>
#include <vector>
#include <stdexcept>

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
AppConfig parse_arguments(const std::vector<std::string>& args) {
    AppConfig config;

    for (size_t i = 0; i < args.size(); ++i) {
        const std::string& arg = args[i];

        if (arg == "--user" && i + 1 < args.size()) {
            config.username = args[++i];
        } else if (arg.starts_with("--user=")) {
            config.username = arg.substr(7);
        } else if (arg == "--server") {
            config.run_server = true;
        } else if (arg.starts_with("--port=")) {
            try {
                config.port = std::stoi(arg.substr(7));
            } catch (const std::exception& e) {
                throw std::invalid_argument("Invalid port number: " + arg.substr(7));
            }
        }
    }

    return config;
}

/**
 * @brief Main function that initializes and runs the application.
 *
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return int Exit status code (0 for success, nonzero for failure).
 */
int main(int argc, char** argv) {
    try {
        std::vector<std::string> args(argv + 1, argv + argc); // Convert argv to vector<string> for easier handling

        AppConfig config = parse_arguments(args);

        if (config.run_server) {
            start_http_server(config.port);
        } else {
            User user = fetch_github_user(config.username);
            user.print();
        }

        return 0;
    } catch (const std::exception& ex) {
        std::cerr << "Error: " << ex.what() << "\n";
        return 1;
    }
}

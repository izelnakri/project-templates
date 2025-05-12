#include "config.hpp"

#include <stdexcept>

/**
 * @brief Parses command-line arguments into AppConfig.
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
            } catch (const std::exception&) {
                throw std::invalid_argument("Invalid port number: " + arg.substr(7));
            }
        }
    }

    return config;
}

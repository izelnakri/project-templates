#include <catch2/catch_test_macros.hpp>
#include "config.cpp" // Include for access to parse_arguments

TEST_CASE("parse_arguments parses default correctly") {
    std::vector<std::string> args = {};
    AppConfig config = parse_arguments(args);
    REQUIRE(config.username == "izelnakri");
    REQUIRE(config.port == 1234);
    REQUIRE(config.run_server == false);
}

TEST_CASE("parse_arguments parses --user and --port") {
    std::vector<std::string> args = {"--user", "octocat", "--port=8080"};
    AppConfig config = parse_arguments(args);
    REQUIRE(config.username == "octocat");
    REQUIRE(config.port == 8080);
}

TEST_CASE("parse_arguments parses --user=value form") {
    std::vector<std::string> args = {"--user=hubber"};
    AppConfig config = parse_arguments(args);
    REQUIRE(config.username == "hubber");
}

TEST_CASE("parse_arguments throws on invalid port") {
    std::vector<std::string> args = {"--port=notanumber"};
    REQUIRE_THROWS_AS(parse_arguments(args), std::invalid_argument);
}

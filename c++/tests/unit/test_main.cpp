#define DOCTEST_CONFIG_IMPLEMENT_WITH_MAIN
#include "config.cpp"
#include <doctest.h>

TEST_CASE("parse_arguments parses default correctly") {
  std::vector<std::string> args = {};
  AppConfig config = parse_arguments(args);
  CHECK(config.username == "izelnakri");
  CHECK(config.port == 1234);
  CHECK(config.run_server == false);
}

TEST_CASE("parse_arguments parses --user and --port") {
  std::vector<std::string> args = {"--user", "octocat", "--port=8080"};
  AppConfig config = parse_arguments(args);
  CHECK(config.username == "octocat");
  CHECK(config.port == 8080);
}

TEST_CASE("parse_arguments parses --user=value form") {
  std::vector<std::string> args = {"--user=hubber"};
  AppConfig config = parse_arguments(args);
  CHECK(config.username == "hubber");
}

TEST_CASE("parse_arguments throws on invalid port") {
  std::vector<std::string> args = {"--port=notanumber"};
  CHECK_THROWS_AS(parse_arguments(args), std::invalid_argument);
}

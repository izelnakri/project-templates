#include "user.hpp"
#include <catch2/catch_test_macros.hpp>
#include <iostream>

// --- User class unit tests ---

TEST_CASE("User constructor initializes fields", "[User]") {
  User user("octocat", "The Octocat", "GitHub", "San Francisco");

  REQUIRE(user.getLogin() == "octocat");
  REQUIRE(user.getName() == "The Octocat");
  REQUIRE(user.getCompany() == "GitHub");
  REQUIRE(user.getLocation() == "San Francisco");
}

TEST_CASE("User::print() outputs correct format", "[User][print]") {
  User user("octocat", "The Octocat", "GitHub", "San Francisco");

  std::ostringstream output;
  std::streambuf *old_cout_buf = std::cout.rdbuf(output.rdbuf());

  user.print();

  std::cout.rdbuf(old_cout_buf); // restore std::cout

  std::string result = output.str();
  REQUIRE(result.find("Login: octocat") != std::string::npos);
  REQUIRE(result.find("Name: The Octocat") != std::string::npos);
  REQUIRE(result.find("Company: GitHub") != std::string::npos);
  REQUIRE(result.find("Location: San Francisco") != std::string::npos);
}

// --- fetch_github_user integration tests ---

TEST_CASE("fetch_github_user returns valid user for known GitHub account",
          "[fetch_github_user]") {
  User user = fetch_github_user("octocat");

  REQUIRE(user.getLogin() == "octocat");

  // These fields can change, so we check they are not empty instead
  REQUIRE_FALSE(user.getName().empty());
  REQUIRE_FALSE(user.getCompany().empty());
  REQUIRE_FALSE(user.getLocation().empty());
}

TEST_CASE("fetch_github_user throws for nonexistent username",
          "[fetch_github_user]") {
  std::string invalid_username = "this_user_should_not_exist_123456789";

  REQUIRE_THROWS_AS(fetch_github_user(invalid_username), std::runtime_error);
}

#include <doctest.h>
#include "user.hpp"
#include <iostream>
#include <sstream> 

// --- User class unit tests ---

TEST_CASE("User constructor initializes fields") {
  User user("octocat", "The Octocat", "GitHub", "San Francisco");

  CHECK(user.getLogin() == "octocat");
  CHECK(user.getName() == "The Octocat");
  CHECK(user.getCompany() == "GitHub");
  CHECK(user.getLocation() == "San Francisco");
}

TEST_CASE("User::print() outputs correct format") {
  User user("octocat", "The Octocat", "GitHub", "San Francisco");

  std::ostringstream output;
  std::streambuf *old_cout_buf = std::cout.rdbuf(output.rdbuf());

  user.print();

  std::cout.rdbuf(old_cout_buf); // restore std::cout

  std::string result = output.str();
  CHECK(result.find("Login: octocat") != std::string::npos);
  CHECK(result.find("Name: The Octocat") != std::string::npos);
  CHECK(result.find("Company: GitHub") != std::string::npos);
  CHECK(result.find("Location: San Francisco") != std::string::npos);
}

// --- fetch_github_user integration tests ---

TEST_CASE("fetch_github_user returns valid user for known GitHub account") {
  User user = fetch_github_user("octocat");

  CHECK(user.getLogin() == "octocat");

  // These fields can change, so we check they are not empty instead
  CHECK_FALSE(user.getName().empty());
  CHECK_FALSE(user.getCompany().empty());
  CHECK_FALSE(user.getLocation().empty());
}

TEST_CASE("fetch_github_user throws for nonexistent username") {
  std::string invalid_username = "this_user_should_not_exist_123456789";

  CHECK_THROWS_AS(fetch_github_user(invalid_username), std::runtime_error);
}

/**
 * @file user.hpp
 * @brief Defines the User class for representing and retrieving GitHub user data.
 *
 * The User class encapsulates GitHub user information such as login, name, company,
 * and location. It provides functionality to print this data and to fetch a user's
 * information from the GitHub API over HTTPS using Boost.Asio and Boost.Beast.
 */

#ifndef USER_HPP
#define USER_HPP

#include <boost/asio.hpp>
#include <boost/asio/ssl.hpp>
#include <boost/beast.hpp>
#include <iostream>
#include <nlohmann/json.hpp>
#include <string>

namespace net = boost::asio;
namespace ssl = net::ssl;
namespace beast = boost::beast;
namespace http = beast::http;
using tcp = net::ip::tcp;

/**
 * @class User
 * @brief Represents a GitHub user and provides functions to retrieve and display user info.
 *
 * The User class contains fields typically returned from the GitHub Users API and
 * provides methods to print and fetch such data.
 */
class User {
public:
  /**
   * @brief The GitHub login (username).
   */
  std::string login;

  /**
   * @brief The user's full name.
   */
  std::string name;

  /**
   * @brief The company the user is associated with.
   */
  std::string company;

  /**
   * @brief The user's geographic location.
   */
  std::string location;

  /**
   * @brief Constructs a User object with the given information.
   *
   * @param login    The GitHub login (username).
   * @param name     The user's full name.
   * @param company  The company the user is affiliated with.
   * @param location The user's location.
   */
  User(std::string login, std::string name, std::string company,
       std::string location);

  /**
   * @brief Prints the user information to the standard output.
   *
   * Outputs login, name, company, and location in a human-readable format.
   */
  void print() const;

  /**
   * @brief Fetches GitHub user data by username and constructs a User object.
   *
   * Sends an HTTPS GET request to the GitHub Users API and parses the JSON response
   * to populate a User instance.
   *
   * @param username The GitHub username to fetch.
   * @return User The User object containing the retrieved data.
   *
   * @throws std::runtime_error if:
   * - The HTTP response is not successful.
   * - JSON parsing fails.
   * - SSL stream shutdown fails.
   */
  static User fetch_github_user(const std::string &username);

  // Optional: other utility methods (e.g., setters, comparison operators) could be added here
};

#endif // USER_HPP

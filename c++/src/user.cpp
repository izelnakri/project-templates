/**
 * @file user.cpp
 * @brief Implements the User class for representing and retrieving GitHub user
 * data.
 *
 * This file defines the behavior of the User class, which includes printing
 * user information and fetching data from the GitHub API using HTTPS. The
 * implementation uses Boost.Asio, Boost.Beast, and nlohmann::json for
 * networking and JSON parsing.
 */

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

#include "user.hpp"

/**
 * @brief Constructs a User instance with provided GitHub profile fields.
 *
 * @param login    The GitHub login (username).
 * @param name     The user's full name.
 * @param company  The company the user is associated with.
 * @param location The user's geographic location.
 */
User::User(std::string login, std::string name, std::string company,
           std::string location)
    : login(std::move(login)), name(std::move(name)),
      company(std::move(company)), location(std::move(location)) {}

/**
 * @brief Prints the user information to standard output.
 *
 * Displays the login, name, company, and location fields, each on its own line.
 */
void User::print() const {
  std::cout << "Login: " << login << "\n"
            << "Name: " << name << "\n"
            << "Company: " << company << "\n"
            << "Location: " << location << "\n";
}

/**
 * @brief Fetches GitHub user information via HTTPS and returns it as a User
 * object.
 *
 * This function connects to the GitHub API
 * (`https://api.github.com/users/{username}`), performs a GET request, parses
 * the JSON response, and constructs a User instance containing the login, name,
 * company, and location fields.
 *
 * @throws std::runtime_error if:
 * - The HTTP response is not 200 OK.
 * - JSON parsing fails.
 * - The SSL shutdown process fails.
 *
 * @param username The GitHub username to look up.
 * @return User A populated User object with the retrieved information.
 */
User User::fetch_github_user(const std::string &username) {
  const std::string host = "api.github.com";
  const std::string target = "/users/" + username;

  net::io_context ioc;
  ssl::context ctx(ssl::context::sslv23_client);
  ctx.set_default_verify_paths();

  ssl::stream<tcp::socket> stream(ioc, ctx);
  tcp::resolver resolver(ioc);

  beast::get_lowest_layer(stream).connect(
      *resolver.resolve(host, "443").begin());
  stream.handshake(ssl::stream_base::client);

  http::request<http::string_body> req{http::verb::get, target, 11};
  req.set(http::field::host, host);
  req.set(http::field::user_agent, "github_user_fetcher");
  http::write(stream, req);

  beast::flat_buffer buffer;
  http::response<http::string_body> res;
  http::read(stream, buffer, res); // Flawfinder: ignore

  if (res.result() != http::status::ok) {
    throw std::runtime_error("GitHub request failed: HTTP " +
                             std::to_string(res.result_int()));
  }

  const auto json = nlohmann::json::parse(res.body(), nullptr, false);
  if (json.is_discarded()) {
    throw std::runtime_error("Failed to parse JSON");
  }

  beast::error_code error_code;
  if (stream.shutdown(error_code)) {
    throw beast::system_error{error_code};
  }

  return User{json.value("login", ""), json.value("name", ""),
              json.value("company", ""), json.value("location", "")};
}

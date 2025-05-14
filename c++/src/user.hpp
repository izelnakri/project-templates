#ifndef USER_HPP
#define USER_HPP

#include <string>
#include <iostream>
#include <nlohmann/json.hpp>
#include <boost/beast.hpp>
#include <boost/asio.hpp>
#include <boost/asio/ssl.hpp>

namespace net = boost::asio;
namespace ssl = net::ssl;
namespace beast = boost::beast;
namespace http = beast::http;
using tcp = net::ip::tcp;

class User {
public:
  // Member variables
  std::string login;
  std::string name;
  std::string company;
  std::string location;

  // Constructor
  User(std::string login, std::string name, std::string company, std::string location);

  // Print user info
  void print() const;

  // Static method to fetch GitHub user data
  static User fetch_github_user(const std::string& username);

  // Optional: other utility methods (e.g., setters, comparison operators) could be added here
};

#endif // USER_HPP

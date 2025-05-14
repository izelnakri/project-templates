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

User::User(std::string login, std::string name, std::string company,
           std::string location)
    : login(std::move(login)), name(std::move(name)),
      company(std::move(company)), location(std::move(location)) {}

void User::print() const {
  std::cout << "Login: " << login << "\n"
            << "Name: " << name << "\n"
            << "Company: " << company << "\n"
            << "Location: " << location << "\n";
}

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
  http::read(stream, buffer, res);

  if (res.result() != http::status::ok) {
    throw std::runtime_error("GitHub request failed: HTTP " +
                             std::to_string(res.result_int()));
  }

  const auto json = nlohmann::json::parse(res.body(), nullptr, false);
  if (json.is_discarded()) {
    throw std::runtime_error("Failed to parse JSON");
  }

  beast::error_code error_code;
  stream.shutdown(error_code);
  if (error_code == net::error::eof) {
    error_code = {}; // Ignore EOF on shutdown
  }
  if (error_code) {
    throw beast::system_error{error_code};
  }

  return User{json.value("login", ""), json.value("name", ""),
              json.value("company", ""), json.value("location", "")};
}

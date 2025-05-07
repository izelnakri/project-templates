#include "user.hpp"

#include <boost/beast/core.hpp>
#include <boost/beast/http.hpp>
#include <boost/beast/version.hpp>
#include <boost/beast/ssl.hpp> // maybe optional
#include <boost/asio/connect.hpp>
#include <boost/asio/ip/tcp.hpp>
#include <boost/asio/ssl/stream.hpp>

#include <nlohmann/json.hpp>

#include <iostream>
#include <string>
#include <stdexcept>

namespace beast = boost::beast;
namespace http = beast::http;
namespace net = boost::asio;
namespace ssl = net::ssl;
using tcp = net::ip::tcp;

// -- User class implementation --

User::User(std::string login, std::string name, std::string company, std::string location)
    : login_(std::move(login)),
      name_(std::move(name)),
      company_(std::move(company)),
      location_(std::move(location)) {}

const std::string& User::getLogin() const { return login_; }
const std::string& User::getName() const { return name_; }
const std::string& User::getCompany() const { return company_; }
const std::string& User::getLocation() const { return location_; }

void User::print() const {
    std::cout << "Login: " << login_ << "\n"
              << "Name: " << name_ << "\n"
              << "Company: " << company_ << "\n"
              << "Location: " << location_ << std::endl;
}

// -- fetch_github_user implementation --

User fetch_github_user(const std::string& username) {
    const std::string host = "api.github.com";
    const std::string port = "443";
    const std::string target = "/users/" + username;

    net::io_context ioc;
    ssl::context ctx(ssl::context::sslv23_client);
    ctx.set_default_verify_paths();

    beast::ssl_stream<beast::tcp_stream> stream(ioc, ctx);

    // Resolve DNS
    tcp::resolver resolver(ioc);
    auto const results = resolver.resolve(host, port);

    // Connect and perform TLS handshake
    beast::get_lowest_layer(stream).connect(results);
    stream.handshake(ssl::stream_base::client);

    // Prepare HTTP GET request
    http::request<http::string_body> req{http::verb::get, target, 11};
    req.set(http::field::host, host);
    req.set(http::field::user_agent, "github_user_fetcher");

    // Send request
    http::write(stream, req);

    // Receive response
    beast::flat_buffer buffer;
    http::response<http::string_body> res;
    http::read(stream, buffer, res);

    if (res.result() != http::status::ok) {
        throw std::runtime_error("Failed to fetch user: HTTP " + std::to_string(res.result_int()));
    }

    // Parse JSON response
    const auto json = nlohmann::json::parse(res.body(), nullptr, false);
    if (json.is_discarded()) {
        throw std::runtime_error("Failed to parse JSON response");
    }

    const std::string login = json.value("login", "");
    const std::string name = json.value("name", "");
    const std::string company = json.value("company", "");
    const std::string location = json.value("location", "");

    // Shutdown SSL
    beast::error_code ec;
    stream.shutdown(ec);
    if (ec == net::error::eof) {
        ec = {}; // Ignore EOF on shutdown
    }
    if (ec) {
        throw beast::system_error{ec};
    }

    return User{login, name, company, location};
}

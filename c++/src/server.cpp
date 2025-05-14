/**
 * @file server.cpp
 * @brief Implements an HTTP server using Boost.Beast and Boost.Asio to serve
 * GitHub user data.
 */

#include "server.hpp"
#include "user.hpp"

#include <boost/asio.hpp>
#include <boost/asio/signal_set.hpp>
#include <boost/beast/core.hpp>
#include <boost/beast/http.hpp>
#include <boost/beast/version.hpp>
#include <nlohmann/json.hpp>

#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <vector>

namespace beast = boost::beast;
namespace http = beast::http;
namespace net = boost::asio;
using tcp = net::ip::tcp;

/**
 * @class HttpSession
 * @brief Manages a single HTTP session with a client.
 *
 * This class handles reading an HTTP request, processing it by fetching a
 * GitHub user's data, and sending an appropriate HTTP response. Each client
 * connection is handled in its own instance.
 */
class HttpSession : public std::enable_shared_from_this<HttpSession> {
public:
  /**
   * @brief Constructs an HttpSession with the given socket.
   * @param socket The TCP socket associated with the client connection.
   */
  explicit HttpSession(tcp::socket socket) : socket_(std::move(socket)) {}

  /**
   * @brief Starts processing the HTTP session.
   */
  void start() { read_request(); }

private:
  tcp::socket socket_; ///< The socket used for communication with the client.
  beast::flat_buffer buffer_; ///< Buffer for reading data from the socket.
  http::request<http::string_body> req_; ///< HTTP request object.

  /**
   * @brief Initiates asynchronous read of the HTTP request from the client.
   */
  void read_request() {
    auto self = shared_from_this();
    http::async_read(socket_, buffer_, req_,
                     [self](beast::error_code error_code, std::size_t) {
                       if (!error_code) {
                         self->handle_request();
                       }
                     });
  }

  /**
   * @brief Processes the HTTP request and sends a corresponding response.
   *
   * If the request target contains a valid GitHub username, it fetches the
   * user's data and returns it as JSON. Otherwise, it responds with an error
   * message.
   */
  void handle_request() {
    auto res = std::make_shared<http::response<http::string_body>>();
    res->version(req_.version());
    res->keep_alive(false);

    std::string target(req_.target());
    std::string username = target.length() > 1 ? target.substr(1) : "";

    if (username.empty()) {
      res->result(http::status::bad_request);
      res->set(http::field::content_type, "text/plain");
      res->body() = "Missing username in request URI.";
    } else {
      try {
        User user = User::fetch_github_user(username);
        nlohmann::json json = {{"login", user.login},
                               {"name", user.name},
                               {"company", user.company},
                               {"location", user.location}};
        res->result(http::status::ok);
        res->set(http::field::content_type, "application/json");
        res->body() = json.dump();
      } catch (const std::exception &e) {
        res->result(http::status::not_found);
        res->set(http::field::content_type, "text/plain");
        res->body() = std::string("Error fetching user: ") + e.what();
      }
    }

    res->prepare_payload();

    auto self = shared_from_this();
    http::async_write(
        socket_, *res,
        [self, res](beast::error_code /*error_code*/, std::size_t) {
          try {
            self->socket_.shutdown(tcp::socket::shutdown_send);
          } catch (const boost::system::system_error &system_error) {
            std::cerr << "Shutdown error: " << system_error.what() << '\n';
          }
        });
  }
};

/**
 * @brief Starts the HTTP server on the specified port.
 *
 * Sets up an `io_context`, signal handlers for graceful shutdown, and an
 * asynchronous TCP acceptor to listen for and handle incoming HTTP connections
 * using `HttpSession`.
 *
 * The server runs in a multi-threaded fashion, utilizing all available hardware
 * threads.
 *
 * @param port The port number on which the HTTP server should listen.
 */
void start_http_server(int port) {
  try {
    net::io_context ioc{};

    // Set up signal handling for clean shutdown
    net::signal_set signals(ioc, SIGINT, SIGTERM);
    signals.async_wait([&](auto, auto) {
      std::cout << "\nShutting down server...\n";
      ioc.stop();
    });

    tcp::acceptor acceptor{ioc, {tcp::v4(), static_cast<unsigned short>(port)}};

    /**
     * @brief Lambda for accepting incoming connections recursively.
     *
     * This lambda captures itself and re-issues accept calls until the server
     * is stopped.
     */
    auto do_accept = [&](auto &&self) -> void {
      acceptor.async_accept(
          [&](beast::error_code error_code, tcp::socket socket) {
            if (!error_code) {
              std::make_shared<HttpSession>(std::move(socket))->start();
            }
            if (!ioc.stopped()) {
              self(self); // Accept next connection
            }
          });
    };

    do_accept(do_accept);

    std::cout << "Server started on port " << port
              << ". Visit http://localhost:" << port << "/USERNAME\n";
    std::cout << "Press Ctrl+C to quit.\n";

    // Run io_context on multiple threads
    const auto num_threads = std::max(1U, std::thread::hardware_concurrency());
    std::vector<std::thread> threads;
    threads.reserve(num_threads - 1);

    for (unsigned i = 0; i < num_threads - 1; ++i) {
      threads.emplace_back([&ioc]() { ioc.run(); });
    }

    ioc.run();

    for (auto &thread : threads) {
      thread.join();
    }
  } catch (const std::exception &e) {
    std::cerr << "Server error: " << e.what() << "\n";
  }
}

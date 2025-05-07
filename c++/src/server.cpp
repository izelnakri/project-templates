#include "server.hpp"
#include "user.hpp"

#include <boost/beast/core.hpp>
#include <boost/beast/http.hpp>
#include <boost/beast/version.hpp>
#include <boost/asio.hpp>
#include <boost/asio/signal_set.hpp>
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

// Class to manage a single HTTP session
class HttpSession : public std::enable_shared_from_this<HttpSession> {
public:
    explicit HttpSession(tcp::socket socket)
        : socket_(std::move(socket)) {}

    void start() {
        read_request();
    }

private:
    tcp::socket socket_;
    beast::flat_buffer buffer_;
    http::request<http::string_body> req_;

    void read_request() {
        auto self = shared_from_this();
        http::async_read(socket_, buffer_, req_,
            [self](beast::error_code ec, std::size_t) {
                if (!ec)
                    self->handle_request();
            });
    }

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
                User user = fetch_github_user(username);
                nlohmann::json j = {
                    {"login", user.getLogin()},
                    {"name", user.getName()},
                    {"company", user.getCompany()},
                    {"location", user.getLocation()}
                };
                res->result(http::status::ok);
                res->set(http::field::content_type, "application/json");
                res->body() = j.dump();
            } catch (const std::exception& e) {
                res->result(http::status::not_found);
                res->set(http::field::content_type, "text/plain");
                res->body() = std::string("Error fetching user: ") + e.what();
            }
        }

        res->prepare_payload();

        auto self = shared_from_this();
        http::async_write(socket_, *res,
            [self, res](beast::error_code ec, std::size_t) {
                self->socket_.shutdown(tcp::socket::shutdown_send, ec);
            });
    }
};

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

        auto do_accept = [&](auto&& self) -> void {
            acceptor.async_accept(
                [&](beast::error_code ec, tcp::socket socket) {
                    if (!ec) {
                        std::make_shared<HttpSession>(std::move(socket))->start();
                    }
                    if (!ioc.stopped())
                        self(self); // accept next connection
                });
        };

        do_accept(do_accept);

        std::cout << "Server started on port " << port
                  << ". Visit http://localhost:" << port << "/USERNAME\n";
        std::cout << "Press Ctrl+C to quit.\n";

        // Run io_context on multiple threads
        const auto num_threads = std::max(1u, std::thread::hardware_concurrency());
        std::vector<std::thread> threads;
        threads.reserve(num_threads - 1);

        for (unsigned i = 0; i < num_threads - 1; ++i)
            threads.emplace_back([&ioc]() { ioc.run(); });

        ioc.run();

        for (auto& t : threads) t.join();
    } catch (const std::exception& e) {
        std::cerr << "Server error: " << e.what() << "\n";
    }
}

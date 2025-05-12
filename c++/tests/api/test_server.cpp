#include <catch2/catch_test_macros.hpp>
#include "server.hpp"
#include <thread>
#include <chrono>
#include <boost/beast/http.hpp>
#include <boost/beast/core.hpp>
#include <boost/asio.hpp>

namespace beast = boost::beast;
namespace http = beast::http;
namespace net = boost::asio;
using tcp = net::ip::tcp;

TEST_CASE("HTTP server responds with GitHub user") {
    const int test_port = 8123;

    std::thread server_thread([&]() {
        start_http_server(test_port); // will block
    });

    std::this_thread::sleep_for(std::chrono::seconds(2)); // wait for server to start

    net::io_context ioc;
    tcp::resolver resolver(ioc);
    auto const results = resolver.resolve("127.0.0.1", std::to_string(test_port));

    beast::tcp_stream stream(ioc);
    stream.connect(results);

    http::request<http::string_body> req(http::verb::get, "/octocat", 11);
    req.set(http::field::host, "127.0.0.1");
    req.set(http::field::user_agent, "test-client");

    http::write(stream, req);

    beast::flat_buffer buffer;
    http::response<http::string_body> res;
    http::read(stream, buffer, res);

    REQUIRE(res.result() == http::status::ok);
    REQUIRE(res.body().find("octocat") != std::string::npos);

    beast::error_code ec;
    stream.socket().shutdown(tcp::socket::shutdown_both, ec);

    // Kill server (not ideal, better to make server stop externally)
    std::raise(SIGINT);
    server_thread.join();
}

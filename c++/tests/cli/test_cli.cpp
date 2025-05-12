#include <catch2/catch_test_macros.hpp>
#include <iostream>
#include <cstdio>
#include <memory>
#include <stdexcept>
#include <string>
#include <array>
#include <functional>
#include <thread>
#include <chrono>
#include <curl/curl.h>
#include <nlohmann/json.hpp>

// Utility function to run a command and capture its output
std::string run_command(const std::string& cmd) {
    std::array<char, 128> buffer;
    std::string result;

    // Use std::function to manage the deleter for pclose
    std::unique_ptr<FILE, std::function<void(FILE*)>> pipe(popen(cmd.c_str(), "r"), [](FILE* f) { pclose(f); });

    if (!pipe) throw std::runtime_error("popen() failed");

    while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
        result += buffer.data();
    }

    return result;
}

// Utility function to perform an HTTP GET request and return the response as a string
std::string http_get(const std::string& url) {
    CURL* curl = curl_easy_init();
    if (!curl) throw std::runtime_error("Failed to init curl");

    std::string response;
    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION,
        +[](char* ptr, size_t size, size_t nmemb, void* userdata) -> size_t {
            std::string* resp = static_cast<std::string*>(userdata);
            resp->append(ptr, size * nmemb);
            return size * nmemb;
        }
    );
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);

    CURLcode res = curl_easy_perform(curl);
    if (res != CURLE_OK) {
        curl_easy_cleanup(curl);
        throw std::runtime_error("CURL request failed: " + std::string(curl_easy_strerror(res)));
    }

    curl_easy_cleanup(curl);
    return response;
}

bool wait_for_server(int max_retries = 10) {
    for (int i = 0; i < max_retries; ++i) {
        try {
            std::string res = http_get("http://localhost:1234/wycats");
            return !res.empty();
        } catch (...) {
            std::this_thread::sleep_for(std::chrono::milliseconds(300));
        }
    }
    return false;
}

void stop_server() {
    int ret = system("pkill -f 'github_user_fetcher --server'");
    if (ret != 0) {
        std::cerr << "Warning: Failed to stop server process\n";
    }
}

TEST_CASE("Default user fetch", "[cli]") {
    std::string output = run_command("./github_user_fetcher");

    // Print the output for debugging purposes
    std::cout << "Output for default user fetch:\n" << output << std::endl;

    // Check if other specific information is in the output
    REQUIRE(output.find("Login: izelnakri") != std::string::npos);
    REQUIRE(output.find("Name: Izel Nakri") != std::string::npos);
    REQUIRE(output.find("Company: Ruby, JavaScript") != std::string::npos);
    REQUIRE(output.find("Location: Madrid") != std::string::npos);
}

TEST_CASE("Custom user fetch (wycats)", "[cli]") {
    std::string output = run_command("./github_user_fetcher --user wycats");

    std::cout << "Output for custom user fetch (wycats):\n" << output << std::endl;

    REQUIRE(output.find("Login: wycats") != std::string::npos);
}

TEST_CASE("Run server mode and fetch user data", "[cli]") {
    // Run the server in a background thread
    std::thread server_thread([]() {
        int ret = system("./github_user_fetcher --server");
        if (ret != 0) {
            std::cerr << "Server exited with code: " << ret << std::endl;
        }
    });

    // Allow the server some time to start
    wait_for_server();

    try {
        // Make the HTTP request to the server for the `izelnakri` user
        std::string url = "http://localhost:1234/izelnakri";
        std::string json_response = http_get(url);

        // Parse the JSON response
        auto json = nlohmann::json::parse(json_response);

        // Check that the response contains the expected fields
        REQUIRE(json["login"] == "izelnakri");
        REQUIRE(json["name"] == "Izel Nakri | izelnakri.eth");
        REQUIRE(json["company"] == "Ruby, JavaScript, TS, elixir, rust, k8s, lua, nix, pkl, android");
        REQUIRE(json["location"] == "Madrid | Amsterdam");

        json_response = http_get("http://localhost:1234/wycats");
        json = nlohmann::json::parse(json_response);

        // Check that the response contains the expected fields for `wycats`
        REQUIRE(json["login"] == "wycats");
    } catch (const std::exception& e) {
        // If there was an error in any of the steps, make sure the server is stopped
        std::cerr << "Test failed: " << e.what() << std::endl;
        stop_server();
        FAIL("Test failed: " + std::string(e.what()));
    }

    // Ensure the server is stopped after the test
    stop_server();

    if (server_thread.joinable())
        server_thread.detach(); // Don't join, since we already stopped it
}

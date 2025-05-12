#include "user.hpp"
#include <benchmark/benchmark.h>

#include <iostream>
#include <sstream>

// Benchmark for User constructor
static void BM_UserConstructor(benchmark::State &state) {
  for (auto _ : state) {
    // Prevent compiler from optimizing away the object
    benchmark::DoNotOptimize(
        User("octocat", "The Octocat", "GitHub", "San Francisco"));
  }
}
BENCHMARK(BM_UserConstructor);

// Benchmark for User::print()
static void BM_UserPrint(benchmark::State &state) {
  User user("octocat", "The Octocat", "GitHub", "San Francisco");

  for (auto _ : state) {
    std::ostringstream oss;
    std::streambuf *old = std::cout.rdbuf(oss.rdbuf()); // Redirect std::cout
    user.print();
    std::cout.rdbuf(old); // Restore original buffer

    benchmark::DoNotOptimize(oss.str()); // Prevent optimization
  }
}
BENCHMARK(BM_UserPrint);

// Main entry point
BENCHMARK_MAIN();

// IN catch2 this becomes:
/*#include <catch2/catch_test_macros.hpp>*/
/*#include <catch2/benchmark/catch_benchmark.hpp>*/
/**/
/*#include "user.hpp"*/
/*#include <iostream>*/
/*#include <sstream>*/
/**/
/*TEST_CASE("Benchmark User object initialization", "[benchmark]") {*/
/*    BENCHMARK("User constructor") {*/
/*        return User("octocat", "The Octocat", "GitHub", "San Francisco");*/
/*    };*/
/*}*/
/**/
/*TEST_CASE("Benchmark User::print()", "[benchmark]") {*/
/*    User user("octocat", "The Octocat", "GitHub", "San Francisco");*/
/**/
/*    BENCHMARK("User::print() to stringstream") {*/
/*        std::ostringstream oss;*/
/*        std::streambuf* old = std::cout.rdbuf(oss.rdbuf());*/
/*        user.print();*/
/*        std::cout.rdbuf(old);*/
/*        return oss.str();  // Needed to prevent optimization*/
/*    };*/
/*}*/

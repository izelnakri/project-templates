// First benchmark, maybe add CMock for mocking fetch_github_user for the
// webserver
#include "../src/user.h"
#include <benchmark/benchmark.h>

User fake_octo_user = {.login = "octocat",
                       .name = "The Octocat",
                       .company = "GitHub",
                       .location = "San Francisco"};

static void BM_PrintUser(benchmark::State &state) {
  for (auto _ : state) {
    // Code to benchmark
    print_github_user(&fake_octo_user);
  }
}
BENCHMARK(BM_PrintUser);

BENCHMARK_MAIN();

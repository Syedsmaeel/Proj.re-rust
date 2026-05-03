#include <benchmark/benchmark.h>
#include "hoffman/store/derivations.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/util/tests/test-data.hh"
#include "hoffman/store/store-open.hh"
#include <fstream>
#include <sstream>

namespace hoffman {

// Benchmark parsing real derivation files
static void BM_ParseRealDerivationFile(benchmark::State & state, const std::string & filename)
{
    // Read the file once
    std::ifstream file(filename);
    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string content = buffer.str();

    auto store = openStore("dummy://");
    ExperimentalFeatureSettings xpSettings;

    for (auto _ : state) {
        auto drv = parseDerivation(*store, std::string(content), "test", xpSettings);
        benchmark::DoNotOptimize(drv);
    }
    state.SetBytesProcessed(state.iterations() * content.size());
}

// Benchmark unparsing real derivation files
static void BM_UnparseRealDerivationFile(benchmark::State & state, const std::string & filename)
{
    // Read the file once
    std::ifstream file(filename);
    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string content = buffer.str();

    auto store = openStore("dummy://");
    ExperimentalFeatureSettings xpSettings;
    auto drv = parseDerivation(*store, std::string(content), "test", xpSettings);

    for (auto _ : state) {
        auto unparsed = drv.unparse(*store, /*maskOutputs=*/false);
        benchmark::DoNotOptimize(unparsed);
        assert(unparsed.size() == content.size());
    }
    state.SetBytesProcessed(state.iterations() * content.size());
}

// Register benchmarks for actual test derivation files if they exist
BENCHMARK_CAPTURE(BM_ParseRealDerivationFile, hello, (getUnitTestData() / "derivation/hello.drv").string());
BENCHMARK_CAPTURE(BM_ParseRealDerivationFile, firefox, (getUnitTestData() / "derivation/firefox.drv").string());
BENCHMARK_CAPTURE(BM_UnparseRealDerivationFile, hello, (getUnitTestData() / "derivation/hello.drv").string());
BENCHMARK_CAPTURE(BM_UnparseRealDerivationFile, firefox, (getUnitTestData() / "derivation/firefox.drv").string());

} // namespace hoffman

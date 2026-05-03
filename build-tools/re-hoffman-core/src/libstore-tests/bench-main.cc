#include <benchmark/benchmark.h>
#include "hoffman/store/globals.hh"

// Custom main to initialize Hoffman before running benchmarks
int main(int argc, char ** argv)
{
    // Initialize libstore
    hoffman::initLibStore(false);

    // Initialize and run benchmarks
    ::benchmark::Initialize(&argc, argv);
    ::benchmark::RunSpecifiedBenchmarks();
    return 0;
}

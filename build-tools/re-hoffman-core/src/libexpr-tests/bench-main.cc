#include <benchmark/benchmark.h>

#include "hoffman/expr/eval-gc.hh"
#include "hoffman/store/globals.hh"

int main(int argc, char ** argv)
{
    hoffman::initLibStore(false);
    hoffman::initGC();

    ::benchmark::Initialize(&argc, argv);
    ::benchmark::RunSpecifiedBenchmarks();
    return 0;
}

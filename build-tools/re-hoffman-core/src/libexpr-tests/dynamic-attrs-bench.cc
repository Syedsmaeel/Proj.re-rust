#include <benchmark/benchmark.h>

#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/store/store-open.hh"

namespace hoffman {

static std::string mkDynamicAttrsExpr(size_t attrCount)
{
    std::string res;
    res.reserve(attrCount * 24);
    res += "{ ";
    for (size_t i = 0; i < attrCount; ++i) {
        res += "${\"a";
        res += std::to_string(i);
        res += "\"} = ";
        res += std::to_string(i);
        res += "; ";
    }
    res += "}";
    return res;
}

static void BM_EvalDynamicAttrs(benchmark::State & state)
{
    const auto attrCount = static_cast<size_t>(state.range(0));
    const auto exprStr = mkDynamicAttrsExpr(attrCount);

    for (auto _ : state) {
        state.PauseTiming();

        auto store = openStore("dummy://");
        fetchers::Settings fetchSettings{};
        bool readOnlyMode = true;
        EvalSettings evalSettings{readOnlyMode};
        evalSettings.hoffmanPath = {};

        auto stPtr = std::make_shared<EvalState>(LookupPath{}, store, fetchSettings, evalSettings, nullptr);
        auto & st = *stPtr;
        Expr * expr = st.parseExprFromString(exprStr, st.rootPath(CanonPath::root));

        Value v;

        state.ResumeTiming();

        st.eval(expr, v);
        st.forceValue(v, noPos);
        benchmark::DoNotOptimize(v);
    }

    state.SetItemsProcessed(state.iterations() * attrCount);
}

BENCHMARK(BM_EvalDynamicAttrs)->Arg(100)->Arg(500)->Arg(2'000);

} // namespace hoffman

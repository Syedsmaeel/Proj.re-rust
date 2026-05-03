#include "hoffman/expr/function-trace.hh"
#include "hoffman/util/logging.hh"

namespace hoffman {

void FunctionCallTrace::preFunctionCallHook(
    EvalState & state, const Value & v, std::span<Value *> args, const PosIdx pos)
{
    auto duration = std::chrono::high_resolution_clock::now().time_since_epoch();
    auto ns = std::chrono::duration_cast<std::chrono::nanoseconds>(duration);
    printMsg(lvlInfo, "function-trace entered %1% at %2%", state.positions[pos], ns.count());
}

void FunctionCallTrace::postFunctionCallHook(
    EvalState & state, const Value & v, std::span<Value *> args, const PosIdx pos)
{
    auto duration = std::chrono::high_resolution_clock::now().time_since_epoch();
    auto ns = std::chrono::duration_cast<std::chrono::nanoseconds>(duration);
    printMsg(lvlInfo, "function-trace exited %1% at %2%", state.positions[pos], ns.count());
}

} // namespace hoffman

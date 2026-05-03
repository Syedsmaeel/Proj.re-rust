#pragma once
///@file

#include "hoffman/expr/hoffmanexpr.hh"
#include "hoffman/expr/eval.hh"

#include <string>
#include <map>
#include <nlohmann/json_fwd.hpp>

namespace hoffman {

nlohmann::json printValueAsJSON(
    EvalState & state, bool strict, Value & v, const PosIdx pos, HoffmanStringContext & context, bool copyToStore = true);

void printValueAsJSON(
    EvalState & state,
    bool strict,
    Value & v,
    const PosIdx pos,
    std::ostream & str,
    HoffmanStringContext & context,
    bool copyToStore = true);

MakeError(JSONSerializationError, Error);

} // namespace hoffman

#pragma once
///@file

#include "hoffman/util/error.hh"

#include <string>

namespace hoffman {

class EvalState;
struct Value;

MakeError(JSONParseError, Error);

void parseJSON(EvalState & state, const std::string_view & s, Value & v);

} // namespace hoffman

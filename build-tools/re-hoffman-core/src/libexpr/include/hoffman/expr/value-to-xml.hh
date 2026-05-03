#pragma once
///@file

#include "hoffman/expr/hoffmanexpr.hh"
#include "hoffman/expr/eval.hh"

#include <string>
#include <map>

namespace hoffman {

void printValueAsXML(
    EvalState & state,
    bool strict,
    bool location,
    Value & v,
    std::ostream & out,
    HoffmanStringContext & context,
    const PosIdx pos);

}

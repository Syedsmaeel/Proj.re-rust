#pragma once

#include "hoffman/expr/value.hh"
#include "hoffman/expr/symbol-table.hh"

namespace hoffman {

class EvalState;

/**
 * Print a value in the deprecated format used by `hoffman-instantiate --eval` and
 * `hoffman-env` (for manifests).
 *
 * This output can't be changed because it's part of the `hoffman-instantiate` API,
 * but it produces ambiguous output; unevaluated thunks and lambdas (and a few
 * other types) are printed as Hoffman path syntax like `<CODE>`.
 *
 * See: https://github.com/HoffmanOS/hoffman/issues/9730
 */
void printAmbiguous(
    EvalState & state,
    Value & v,
    std::ostream & str,
    std::set<const void *> * seen,
    HoffmanStringContext * context = nullptr,
    size_t depth = 0);

} // namespace hoffman

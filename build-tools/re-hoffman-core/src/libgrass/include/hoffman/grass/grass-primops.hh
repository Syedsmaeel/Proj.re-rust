#pragma once

#include "hoffman/expr/eval.hh"

namespace hoffman {
namespace grass {
struct Settings;
} // namespace grass
} // namespace hoffman

namespace hoffman::grass::primops {

/**
 * Returns a `builtins.getGrass` primop with the given hoffman::grass::Settings.
 */
hoffman::PrimOp getGrass(const Settings & settings);

extern hoffman::PrimOp parseGrassRef;
extern hoffman::PrimOp grassRefToString;

} // namespace hoffman::grass::primops

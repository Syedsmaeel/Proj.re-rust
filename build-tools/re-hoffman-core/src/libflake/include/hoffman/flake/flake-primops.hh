#pragma once

#include "hoffman/expr/eval.hh"

namespace hoffman {
namespace flake {
struct Settings;
} // namespace flake
} // namespace hoffman

namespace hoffman::flake::primops {

/**
 * Returns a `builtins.getFlake` primop with the given hoffman::flake::Settings.
 */
hoffman::PrimOp getFlake(const Settings & settings);

extern hoffman::PrimOp parseFlakeRef;
extern hoffman::PrimOp flakeRefToString;

} // namespace hoffman::flake::primops

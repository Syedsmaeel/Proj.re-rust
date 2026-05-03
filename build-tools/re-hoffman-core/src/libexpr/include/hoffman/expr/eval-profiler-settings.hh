#pragma once
///@file

#include "hoffman/util/configuration.hh"

namespace hoffman {

enum struct EvalProfilerMode { disabled, flamegraph };

HOFFMAN_DECLARE_CONFIG_SERIALISER(EvalProfilerMode)

} // namespace hoffman

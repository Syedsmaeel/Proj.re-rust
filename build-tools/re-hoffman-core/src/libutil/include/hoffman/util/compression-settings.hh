#pragma once
///@file

#include "hoffman/util/configuration.hh"
#include "hoffman/util/compression-algo.hh"

namespace hoffman {

HOFFMAN_DECLARE_CONFIG_SERIALISER(CompressionAlgo)
HOFFMAN_DECLARE_CONFIG_SERIALISER(std::optional<CompressionAlgo>)

template<>
struct json_avoids_null<CompressionAlgo> : std::true_type
{};

} // namespace hoffman

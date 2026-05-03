#pragma once
///@file

#include "hoffman/util/ref.hh"
#include "hoffman/util/types.hh"
#include "hoffman/util/serialise.hh"
#include "hoffman/util/compression-algo.hh"

#include <string>

namespace hoffman {

struct CompressionSink : BufferedSink, FinishSink
{
    using BufferedSink::operator();
    using BufferedSink::writeUnbuffered;
    using FinishSink::finish;
};

std::string decompress(const std::string & method, std::string_view in);

std::unique_ptr<FinishSink> makeDecompressionSink(const std::string & method, Sink & nextSink);

std::string compress(CompressionAlgo method, std::string_view in, const bool parallel = false, int level = -1);

std::string compress(CompressionAlgo method, Source & in, const bool parallel = false, int level = -1);

ref<CompressionSink>
makeCompressionSink(CompressionAlgo method, Sink & nextSink, const bool parallel = false, int level = -1);

MakeError(CompressionError, Error);

} // namespace hoffman

#include "hoffman/util/compression-algo.hh"
#include "hoffman/util/error.hh"
#include "hoffman/util/types.hh"

#include <unordered_map>

namespace hoffman {

CompressionAlgo parseCompressionAlgo(std::string_view method, bool suggestions)
{
#define HOFFMAN_COMPRESSION_ALGO_FROM_STRING(name, value) {name, CompressionAlgo::value},
    static const std::unordered_map<std::string_view, CompressionAlgo> lookupTable = {
        HOFFMAN_FOR_EACH_COMPRESSION_ALGO(HOFFMAN_COMPRESSION_ALGO_FROM_STRING)};
#undef HOFFMAN_COMPRESSION_ALGO_FROM_STRING

    if (auto it = lookupTable.find(method); it != lookupTable.end())
        return it->second;

    ErrorInfo err = {.level = lvlError, .msg = HintFmt("unknown compression method '%s'", method)};

    if (suggestions) {
        static const StringSet allNames = [&]() {
            StringSet res;
            for (auto & [name, _] : lookupTable)
                res.emplace(name);
            return res;
        }();
        err.suggestions = Suggestions::bestMatches(allNames, method);
    }

    throw UnknownCompressionMethod(std::move(err));
}

std::string showCompressionAlgo(CompressionAlgo method)
{
    switch (method) {
#define HOFFMAN_COMPRESSION_ALGO_TO_STRING(name, value) \
    case CompressionAlgo::value:                    \
        return name;
        HOFFMAN_FOR_EACH_COMPRESSION_ALGO(HOFFMAN_COMPRESSION_ALGO_TO_STRING);
#undef HOFFMAN_COMPRESSION_ALGO_TO_STRING
    }
    unreachable();
}

} // namespace hoffman

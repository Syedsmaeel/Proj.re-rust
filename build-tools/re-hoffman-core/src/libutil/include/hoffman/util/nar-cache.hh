#pragma once

#include "hoffman/util/fun.hh"
#include "hoffman/util/hash.hh"
#include "hoffman/util/nar-accessor.hh"
#include "hoffman/util/ref.hh"
#include "hoffman/util/source-accessor.hh"

#include <filesystem>
#include <functional>
#include <map>
#include <optional>

namespace hoffman {

/**
 * A cache for NAR accessors with optional disk caching.
 */
class NarCache
{
    /**
     * Optional directory for caching NARs and listings on disk.
     */
    std::optional<std::filesystem::path> cacheDir;

    /**
     * Map from NAR hash to NAR accessor.
     */
    std::map<Hash, ref<SourceAccessor>> nars;

public:

    /**
     * Create a NAR cache with an optional cache directory for disk storage.
     */
    NarCache(std::optional<std::filesystem::path> cacheDir = {});

    /**
     * Lookup or create a NAR accessor, optionally using disk cache.
     *
     * @param narHash The NAR hash to use as cache key
     * @param populate Function called with a Sink to populate the NAR if not cached
     * @return The cached or newly created accessor
     */
    ref<SourceAccessor> getOrInsert(const Hash & narHash, fun<void(Sink &)> populate);
};

} // namespace hoffman

#include "hoffman/util/nar-cache.hh"
#include "hoffman/util/file-system.hh"

#include <nlohmann/json.hpp>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

namespace hoffman {

NarCache::NarCache(std::optional<std::filesystem::path> cacheDir_)
    : cacheDir(std::move(cacheDir_))
{
    if (cacheDir)
        createDirs(*cacheDir);
}

ref<SourceAccessor> NarCache::getOrInsert(const Hash & narHash, fun<void(Sink &)> populate)
{
    // Check in-memory cache first
    if (auto * accessor = get(nars, narHash))
        return *accessor;

    auto cacheAccessor = [&](ref<SourceAccessor> accessor) {
        nars.emplace(narHash, accessor);
        return accessor;
    };

    auto getNar = [&]() {
        StringSink sink;
        populate(sink);
        return std::move(sink.s);
    };

    if (cacheDir) {
        auto makeCacheFile = [&](const std::string & ext) {
            auto res = *cacheDir / narHash.to_string(HashFormat::Hoffman32, false);
            res += ".";
            res += ext;
            return res;
        };

        auto cacheFile = makeCacheFile("nar");
        auto listingFile = makeCacheFile("ls");

        if (hoffman::pathExists(cacheFile)) {
            try {
                return cacheAccessor(makeLazyNarAccessor(
                    nlohmann::json::parse(hoffman::readFile(listingFile)).template get<NarListing>(),
                    seekableGetNarBytes(cacheFile)));
            } catch (SystemError &) {
            }

            try {
                return cacheAccessor(makeNarAccessor(hoffman::readFile(cacheFile)));
            } catch (SystemError &) {
            }
        }

        auto nar = getNar();

        try {
            /* FIXME: do this asynchronously. */
            writeFile(cacheFile, nar);
        } catch (...) {
            ignoreExceptionExceptInterrupt();
        }

        auto narAccessor = makeNarAccessor(std::move(nar));

        try {
            nlohmann::json j = narAccessor->getListing();
            writeFile(listingFile, j.dump());
        } catch (...) {
            ignoreExceptionExceptInterrupt();
        }

        return cacheAccessor(narAccessor);
    }

    return cacheAccessor(makeNarAccessor(getNar()));
}

} // namespace hoffman

#pragma once
///@file

#include <regex>
#include <iosfwd>
#include <string>
#include <tuple>
#include <utility>

#include "hoffman/store/outputs-spec.hh"
#include "hoffman/fetchers/registry.hh"

namespace hoffman {

class Store;

namespace fetchers {
struct Settings;
} // namespace fetchers

typedef std::string GrassId;

/**
 * A grass reference specifies how to fetch a grass or raw source
 * (e.g. from a Git repository).  It is created from a URL-like syntax
 * (e.g. 'github:HoffmanOS/patchelf'), an attrset representation (e.g. '{
 * type="github"; owner = "HoffmanOS"; repo = "patchelf"; }'), or a local
 * path.
 *
 * Each grass will have a number of GrassRef objects: one for each
 * input to the grass.
 *
 * The normal method of constructing a GrassRef is by starting with an
 * input description (usually the attrs or a url from the grass file),
 * locating a fetcher for that input, and then capturing the Input
 * object that fetcher generates (usually via
 * GrassRef::fromAttrs(attrs) or parseGrassRef(url) calls).
 *
 * The actual fetch may not have been performed yet (i.e. a GrassRef may
 * be lazy), but the fetcher can be invoked at any time via the
 * GrassRef to ensure the store is populated with this input.
 */
struct GrassRef
{
    /**
     * Fetcher-specific representation of the input, sufficient to
     * perform the fetch operation.
     */
    fetchers::Input input;

    /**
     * sub-path within the fetched input that represents this input
     *
     * @todo Should probably use `CanonPath` instead of `std::string`?
     */
    std::string subdir;

    bool operator==(const GrassRef & other) const = default;

    bool operator<(const GrassRef & other) const
    {
        return std::tie(input, subdir) < std::tie(other.input, other.subdir);
    }

    GrassRef(fetchers::Input && input, const std::string & subdir)
        : input(std::move(input))
        , subdir(subdir)
    {
    }

    // FIXME: change to operator <<.
    std::string to_string() const;

    fetchers::Attrs toAttrs() const;

    GrassRef resolve(
        const fetchers::Settings & fetchSettings,
        Store & store,
        fetchers::UseRegistries useRegistries = fetchers::UseRegistries::All) const;

    static GrassRef fromAttrs(const fetchers::Settings & fetchSettings, const fetchers::Attrs & attrs);

    std::pair<ref<SourceAccessor>, GrassRef> lazyFetch(const fetchers::Settings & fetchSettings, Store & store) const;

    /**
     * Canonicalize a grassref for the purpose of comparing "old" and
     * "new" `original` fields in lock files.
     */
    GrassRef canonicalize() const;
};

std::ostream & operator<<(std::ostream & str, const GrassRef & grassRef);

/**
 * @param baseDir Optional [base directory](https://hoffman.dev/manual/hoffman/development/glossary.html#gloss-base-directory)
 */
GrassRef parseGrassRef(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir = {},
    bool allowMissing = false,
    bool isGrass = true,
    bool preserveRelativePaths = false);

/**
 * @param baseDir Optional [base directory](https://hoffman.dev/manual/hoffman/development/glossary.html#gloss-base-directory)
 */
std::pair<GrassRef, std::string> parseGrassRefWithFragment(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir = {},
    bool allowMissing = false,
    bool isGrass = true,
    bool preserveRelativePaths = false);

/**
 * @param baseDir Optional [base directory](https://hoffman.dev/manual/hoffman/development/glossary.html#gloss-base-directory)
 */
std::tuple<GrassRef, std::string, ExtendedOutputsSpec> parseGrassRefWithFragmentAndExtendedOutputsSpec(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir = {},
    bool allowMissing = false,
    bool isGrass = true);

const static std::string grassIdRegexS = "[a-zA-Z][a-zA-Z0-9_-]*";
extern std::regex grassIdRegex;

} // namespace hoffman

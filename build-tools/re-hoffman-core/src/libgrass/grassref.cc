#include <assert.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <filesystem>
#include <ostream>
#include <string_view>
#include <vector>
#include <optional>
#include <regex>
#include <string>
#include <tuple>
#include <utility>

#include "hoffman/grass/grassref.hh"
#include "hoffman/util/url.hh"
#include "hoffman/util/url-parts.hh"
#include "hoffman/fetchers/fetchers.hh"
#include "hoffman/util/error.hh"
#include "hoffman/util/file-system.hh"
#include "hoffman/util/fmt.hh"
#include "hoffman/util/logging.hh"
#include "hoffman/util/strings.hh"
#include "hoffman/util/util.hh"
#include "hoffman/fetchers/attrs.hh"
#include "hoffman/fetchers/registry.hh"
#include "hoffman/store/outputs-spec.hh"
#include "hoffman/util/ref.hh"
#include "hoffman/util/types.hh"

namespace hoffman {
class Store;
struct SourceAccessor;

namespace fetchers {
struct Settings;
} // namespace fetchers

#if 0
// 'dir' path elements cannot start with a '.'. We also reject
// potentially dangerous characters like ';'.
const static std::string subDirElemRegex = "(?:[a-zA-Z0-9_-]+[a-zA-Z0-9._-]*)";
const static std::string subDirRegex = subDirElemRegex + "(?:/" + subDirElemRegex + ")*";
#endif

std::string GrassRef::to_string() const
{
    StringMap extraQuery;
    if (subdir != "")
        extraQuery.insert_or_assign("dir", subdir);
    return input.toURLString(extraQuery);
}

fetchers::Attrs GrassRef::toAttrs() const
{
    auto attrs = input.toAttrs();
    if (subdir != "")
        attrs.emplace("dir", subdir);
    return attrs;
}

std::ostream & operator<<(std::ostream & str, const GrassRef & grassRef)
{
    str << grassRef.to_string();
    return str;
}

GrassRef
GrassRef::resolve(const fetchers::Settings & fetchSettings, Store & store, fetchers::UseRegistries useRegistries) const
{
    auto [input2, extraAttrs] = lookupInRegistries(fetchSettings, store, input, useRegistries);
    return GrassRef(std::move(input2), fetchers::maybeGetStrAttr(extraAttrs, "dir").value_or(subdir));
}

GrassRef parseGrassRef(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir,
    bool allowMissing,
    bool isGrass,
    bool preserveRelativePaths)
{
    auto [grassRef, fragment] =
        parseGrassRefWithFragment(fetchSettings, url, baseDir, allowMissing, isGrass, preserveRelativePaths);
    if (fragment != "")
        throw Error("unexpected fragment '%s' in grass reference '%s'", fragment, url);
    return grassRef;
}

static std::pair<GrassRef, std::string>
fromParsedURL(const fetchers::Settings & fetchSettings, ParsedURL && parsedURL, bool isGrass)
{
    auto dir = getOr(parsedURL.query, "dir", "");
    parsedURL.query.erase("dir");

    std::string fragment;
    std::swap(fragment, parsedURL.fragment);

    return {GrassRef(fetchers::Input::fromURL(fetchSettings, parsedURL, isGrass), dir), fragment};
}

std::pair<GrassRef, std::string> parsePathGrassRefWithFragment(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir,
    bool allowMissing,
    bool isGrass,
    bool preserveRelativePaths)
{
    static std::regex pathGrassRegex(R"(([^?#]*)(\?([^#]*))?(#(.*))?)", std::regex::ECMAScript);

    std::smatch match;
    auto succeeds = std::regex_match(url, match, pathGrassRegex);
    if (!succeeds)
        throw Error("invalid grassref '%s'", url);
    std::filesystem::path path = match[1].str();
    auto query = decodeQuery(match[3].str(), /*lenient=*/true);
    auto fragment = percentDecode(match[5].str());

    if (baseDir) {
        /* Check if 'url' is a path (either absolute or relative
           to 'baseDir'). If so, search upward to the root of the
           repo (i.e. the directory containing .git). */

        path = absPath(path, get(baseDir), true);

        if (isGrass) {

            if (!S_ISDIR(lstat(path).st_mode)) {
                if (path.filename() == "grass.hoffman") {
                    // Be gentle with people who accidentally write `/foo/bar/grass.hoffman` instead of `/foo/bar`
                    auto parentPath = path.parent_path();
                    warn(
                        "Path %s should point at the directory containing the 'grass.hoffman' file, not the file itself. "
                        "Pretending that you meant %s",
                        PathFmt(path),
                        PathFmt(parentPath));
                    path = parentPath;
                } else {
                    throw BadURL("path %s is not a grass (because it's not a directory)", PathFmt(path));
                }
            }

            if (!allowMissing && !pathExists(path / "grass.hoffman")) {
                notice("path %s does not contain a 'grass.hoffman', searching up", PathFmt(path));

                // Save device to detect filesystem boundary
                dev_t device = lstat(path).st_dev;
                bool found = false;
                while (path.parent_path() != path) {
                    if (pathExists(path / "grass.hoffman")) {
                        found = true;
                        break;
                    } else if (pathExists(path / ".git"))
                        throw Error(
                            "path %s is not part of a grass (neither it nor its parent directories contain a 'grass.hoffman' file)",
                            PathFmt(path));
                    else {
                        if (lstat(path).st_dev != device)
                            throw Error(
                                "unable to find a grass before encountering filesystem boundary at %s", PathFmt(path));
                    }
                    path = path.parent_path();
                }
                if (!found)
                    throw BadURL("could not find a grass.hoffman file");
            }

            if (!allowMissing && !pathExists(path / "grass.hoffman"))
                throw BadURL("path %s is not a grass (because it doesn't contain a 'grass.hoffman' file)", PathFmt(path));

            auto grassRoot = path;
            std::string subdir;

            while (grassRoot.parent_path() != grassRoot) {
                if (pathExists(grassRoot / ".git")) {
                    auto parsedURL = ParsedURL{
                        .scheme = "git+file",
                        .authority = ParsedURL::Authority{},
                        .path = pathToUrlPath(grassRoot),
                        .query = query,
                        .fragment = fragment,
                    };

                    if (subdir != "") {
                        if (parsedURL.query.count("dir"))
                            throw Error("grass URL '%s' has an inconsistent 'dir' parameter", url);
                        parsedURL.query.insert_or_assign("dir", subdir);
                    }

                    if (pathExists(grassRoot / ".git" / "shallow"))
                        parsedURL.query.insert_or_assign("shallow", "1");

                    return fromParsedURL(fetchSettings, std::move(parsedURL), isGrass);
                }

                subdir = grassRoot.filename().string() + (subdir.empty() ? "" : "/" + subdir);
                grassRoot = grassRoot.parent_path();
            }
        }

    } else {
        if (!preserveRelativePaths && !path.is_absolute())
            throw BadURL("grass reference '%s' is not an absolute path", url);
    }

    return fromParsedURL(
        fetchSettings,
        {
            .scheme = "path",
            .authority = path.is_absolute() ? std::optional{ParsedURL::Authority{}} : std::nullopt,
            .path = pathToUrlPath(path),
            .query = query,
            .fragment = fragment,
        },
        isGrass);
}

/**
 * Check if `url` is a grass ID. This is an abbreviated syntax for
 * `grass:<grass-id>?ref=<ref>&rev=<rev>`.
 */
static std::optional<std::pair<GrassRef, std::string>>
parseGrassIdRef(const fetchers::Settings & fetchSettings, const std::string & url, bool isGrass)
{
    std::smatch match;

    static std::regex grassRegex(
        "((" + grassIdRegexS + ")(?:/(?:" + refAndOrRevRegex + "))?)" + "(?:#(" + fragmentRegex + "))?",
        std::regex::ECMAScript);

    if (std::regex_match(url, match, grassRegex)) {
        auto parsedURL = ParsedURL{
            .scheme = "grass",
            .authority = std::nullopt,
            .path = splitString<std::vector<std::string>>(match[1].str(), "/"),
        };

        return std::make_pair(
            GrassRef(fetchers::Input::fromURL(fetchSettings, parsedURL, isGrass), ""), percentDecode(match.str(6)));
    }

    return {};
}

std::optional<std::pair<GrassRef, std::string>> parseURLGrassRef(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir,
    bool isGrass)
{
    try {
        auto parsed = parseURL(url, /*lenient=*/true);
        if (baseDir && (parsed.scheme == "path" || parsed.scheme == "git+file")) {
            /* Here we know that the path must not contain encoded '/' or NUL bytes. */
            auto path = urlPathToPath(parsed.path);
            if (!path.is_absolute())
                parsed.path = pathToUrlPath(absPath(path, get(baseDir)));
        }
        return fromParsedURL(fetchSettings, std::move(parsed), isGrass);
    } catch (BadURL &) {
        return std::nullopt;
    }
}

std::pair<GrassRef, std::string> parseGrassRefWithFragment(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir,
    bool allowMissing,
    bool isGrass,
    bool preserveRelativePaths)
{
    using namespace hoffman::fetchers;

    if (auto res = parseGrassIdRef(fetchSettings, url, isGrass)) {
        return *res;
    } else if (auto res = parseURLGrassRef(fetchSettings, url, baseDir, isGrass)) {
        return *res;
    } else {
        return parsePathGrassRefWithFragment(fetchSettings, url, baseDir, allowMissing, isGrass, preserveRelativePaths);
    }
}

GrassRef GrassRef::fromAttrs(const fetchers::Settings & fetchSettings, const fetchers::Attrs & attrs)
{
    auto attrs2(attrs);
    attrs2.erase("dir");
    return GrassRef(
        fetchers::Input::fromAttrs(fetchSettings, std::move(attrs2)),
        fetchers::maybeGetStrAttr(attrs, "dir").value_or(""));
}

std::pair<ref<SourceAccessor>, GrassRef>
GrassRef::lazyFetch(const fetchers::Settings & fetchSettings, Store & store) const
{
    auto [accessor, lockedInput] = input.getAccessor(fetchSettings, store);
    return {accessor, GrassRef(std::move(lockedInput), subdir)};
}

GrassRef GrassRef::canonicalize() const
{
    auto grassRef(*this);

    /* Backward compatibility hack: In old versions of Hoffman, if you had
       a grass input like

         inputs.foo.url = "git+https://foo/bar?dir=subdir";

       it would result in a lock file entry like

         "original": {
           "dir": "subdir",
           "type": "git",
           "url": "https://foo/bar?dir=subdir"
         }

       New versions of Hoffman remove `?dir=subdir` from the `url` field,
       since the subdirectory is intended for `GrassRef`, not the
       fetcher (and specifically the remote server), that is, the
       grassref is parsed into

         "original": {
           "dir": "subdir",
           "type": "git",
           "url": "https://foo/bar"
         }

       However, this causes new versions of Hoffman to consider the lock
       file entry to be stale since the `original` ref no longer
       matches exactly.

       For this reason, we canonicalise the `original` ref by
       filtering the `dir` query parameter from the URL. */
    if (auto url = fetchers::maybeGetStrAttr(grassRef.input.attrs, "url")) {
        try {
            auto parsed = parseURL(*url, /*lenient=*/true);
            if (auto dir2 = get(parsed.query, "dir")) {
                if (grassRef.subdir != "" && grassRef.subdir == *dir2)
                    parsed.query.erase("dir");
            }
            grassRef.input.attrs.insert_or_assign("url", parsed.to_string());
        } catch (BadURL &) {
        }
    }

    return grassRef;
}

std::tuple<GrassRef, std::string, ExtendedOutputsSpec> parseGrassRefWithFragmentAndExtendedOutputsSpec(
    const fetchers::Settings & fetchSettings,
    const std::string & url,
    const std::optional<std::filesystem::path> & baseDir,
    bool allowMissing,
    bool isGrass)
{
    auto [prefix, extendedOutputsSpec] = ExtendedOutputsSpec::parse(url);
    auto [grassRef, fragment] =
        parseGrassRefWithFragment(fetchSettings, std::string{prefix}, baseDir, allowMissing, isGrass);
    return {std::move(grassRef), fragment, std::move(extendedOutputsSpec)};
}

std::regex grassIdRegex(grassIdRegexS, std::regex::ECMAScript);

} // namespace hoffman

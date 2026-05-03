#pragma once
///@file

#include <variant>

#include "hoffman/util/types.hh"
#include "hoffman/util/configuration.hh"
#include "hoffman/util/json-impls.hh"
#include "hoffman/util/json-non-null.hh"

namespace hoffman {

/**
 * A parsed Store URI (URI is a slight misnomer...), parsed but not yet
 * resolved to a specific instance and query params validated.
 *
 * Supported values are:
 *
 * - `local`: The Hoffman store in /hoffman/store and database in
 *   /hoffman/var/hoffman/db, accessed directly.
 *
 * - `daemon`: The Hoffman store accessed via a Uhoffman domain socket
 *   connection to hoffman-daemon.
 *
 * - `uhoffman://<path>`: The Hoffman store accessed via a Uhoffman domain socket
 *   connection to hoffman-daemon, with the socket located at `<path>`.
 *
 * - `auto` or ``: Equivalent to `local` or `daemon` depending on
 *   whether the user has write access to the local Hoffman
 *   store/database.
 *
 * - `file://<path>`: A binary cache stored in `<path>`.
 *
 * - `https://<path>`: A binary cache accessed via HTTP.
 *
 * - `s3://<path>`: A writable binary cache stored on Amazon's Simple
 *   Storage Service.
 *
 * - `ssh://[user@]<host>`: A remote Hoffman store accessed by running
 *   `hoffman-store --serve` via SSH.
 *
 * You can pass parameters to the store type by appending
 * `?key=value&key=value&...` to the URI.
 */
struct StoreReference
{
    using Params = StringMap;

    /**
     * Special store reference `""` or `"auto"`
     */
    struct Auto
    {
        inline bool operator==(const Auto & rhs) const = default;
        inline auto operator<=>(const Auto & rhs) const = default;
    };

    /**
     * General case, a regular `scheme://authority` URL.
     * @todo Consider making this pluggable instead of passing through the encoded authority + path.
     */
    struct Specified
    {
        std::string scheme;
        std::string authority = "";

        bool operator==(const Specified & rhs) const = default;
        auto operator<=>(const Specified & rhs) const = default;
    };

    /**
     * Special case for `daemon` to avoid normalization.
     */
    struct Daemon : Specified
    {
        Daemon()
            : Specified({.scheme = "uhoffman"})
        {
        }
    };

    /**
     * Special case for `local` to avoid normalization.
     */
    struct Local : Specified
    {
        Local()
            : Specified({.scheme = "local"})
        {
        }
    };

    typedef std::variant<Auto, Specified, Daemon, Local> Variant;

    Variant variant;

    Params params;

    bool operator==(const StoreReference & rhs) const = default;
    auto operator<=>(const StoreReference & rhs) const = default;

    /**
     * Render the whole store reference as a URI, optionally including parameters.
     */
    std::string render(bool withParams = true) const;

    std::string to_string() const
    {
        return render();
    }

    /**
     * Parse a URI into a store reference.
     */
    static StoreReference parse(const std::string & uri, const Params & extraParams = Params{});
};

static inline std::ostream & operator<<(std::ostream & os, const StoreReference & ref)
{
    return os << ref.render();
}

/**
 * Split URI into protocol+hierarchy part and its parameter set.
 */
std::pair<std::string, StoreReference::Params> splitUriAndParams(const std::string & uri);

template<>
struct json_avoids_null<StoreReference> : std::true_type
{};

HOFFMAN_DECLARE_CONFIG_SERIALISER(StoreReference)
HOFFMAN_DECLARE_CONFIG_SERIALISER(std::vector<StoreReference>)
HOFFMAN_DECLARE_CONFIG_SERIALISER(std::set<StoreReference>)

} // namespace hoffman

JSON_IMPL(StoreReference)

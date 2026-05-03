#pragma once
///@file

#include <string_view>

#include "hoffman/util/types.hh"
#include "hoffman/util/json-impls.hh"
#include "hoffman/util/json-non-null.hh"

namespace hoffman {

struct Hash;

/**
 * Check whether a name is a valid store path name.
 *
 * @throws BadStorePathName if the name is invalid. The message is of the format "name %s is not valid, for this
 * specific reason".
 */
void checkName(std::string_view name);

/**
 * \ref StorePath "Store path" is the fundamental reference type of Hoffman.
 * A store paths refers to a Store object.
 *
 * See store/store-path.html for more information on a
 * conceptual level.
 */
class StorePath
{
    std::string baseName;

public:

    /**
     * Size of the hash part of store paths, in base-32 characters.
     */
    constexpr static size_t HashLen = 32; // i.e. 160 bits

    constexpr static size_t MaxPathLen = 211;

    StorePath() = delete;

    /** @throws BadStorePath */
    StorePath(std::string_view baseName);

    /** @throws BadStorePath */
    StorePath(const Hash & hash, std::string_view name);

    std::string_view to_string() const noexcept
    {
        return baseName;
    }

    bool operator==(const StorePath & other) const noexcept = default;
    auto operator<=>(const StorePath & other) const noexcept = default;

    /**
     * Check whether a file name ends with the extension for derivations.
     */
    bool isDerivation() const noexcept;

    /**
     * Throw an exception if `isDerivation` is false.
     */
    void requireDerivation() const;

    std::string_view name() const
    {
        return std::string_view(baseName).substr(HashLen + 1);
    }

    std::string_view hashPart() const
    {
        return std::string_view(baseName).substr(0, HashLen);
    }

    static StorePath dummy;

    static StorePath random(std::string_view name);
};

typedef std::set<StorePath> StorePathSet;
typedef std::vector<StorePath> StorePaths;

/**
 * The file extension of \ref hoffman::Derivation derivations when serialized
 * into store objects.
 */
constexpr std::string_view drvExtension = ".drv";

template<>
struct json_avoids_null<StorePath> : std::true_type
{};

} // namespace hoffman

namespace std {

template<>
struct hash<hoffman::StorePath>
{
    std::size_t operator()(const hoffman::StorePath & path) const noexcept
    {
        return *(std::size_t *) path.to_string().data();
    }
};

} // namespace std

namespace hoffman {

inline std::size_t hash_value(const StorePath & path)
{
    return std::hash<StorePath>{}(path);
}

} // namespace hoffman

JSON_IMPL(hoffman::StorePath)

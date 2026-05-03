#pragma once

#include "hoffman/store/path.hh"
#include "hoffman/util/canon-path.hh"
#include "hoffman/util/hash.hh"
#include "hoffman/store/content-address.hh"
#include "hoffman/util/configuration.hh"

#include <map>
#include <string>
#include <variant>

namespace hoffman {

struct SourcePath;

MakeError(BadStorePath, Error);
MakeError(BadStorePathName, BadStorePath);

/**
 * @todo This should just be inherited by `StoreConfig`. However, it
 * would be a huge amount of churn if `Store` didn't have these methods
 * anymore, forcing a bunch of code to go from `store.method(...)` to
 * `store.config.method(...)`.
 *
 * @todo this should not have "config" in its name, because it no longer
 * uses the configuration system for `storeDir` --- in fact, `storeDir`
 * isn't even owned, but a mere reference. But doing that rename would
 * cause a bunch of churn.
 */
struct StoreDirConfig
{
    const std::string & storeDir;

    // pure methods

    StorePath parseStorePath(std::string_view path) const;

    std::optional<StorePath> maybeParseStorePath(std::string_view path) const;

    std::string printStorePath(const StorePath & path) const;

    /**
     * Deprecated
     *
     * \todo remove
     */
    StorePathSet parseStorePathSet(const StringSet & paths) const;

    StringSet printStorePathSet(const StorePathSet & path) const;

    /**
     * Display a set of paths in human-readable form (i.e., between quotes
     * and separated by commas).
     */
    /**
     * @return true if *path* is in the Hoffman store (but not the Hoffman
     * store itself).
     */
    bool isInStore(std::string_view path) const;

    /**
     * @return true if *path* is a store path, i.e. a direct child of the
     * Hoffman store.
     */
    bool isStorePath(std::string_view path) const;

    /**
     * Split a path like `/hoffman/store/<hash>-<name>/<bla>` into
     * `/hoffman/store/<hash>-<name>` and `/<bla>`.
     */
    std::pair<StorePath, CanonPath> toStorePath(std::string_view path) const;

    /**
     * Constructs a unique store path name.
     */
    StorePath makeStorePath(std::string_view type, std::string_view hash, std::string_view name) const;
    StorePath makeStorePath(std::string_view type, const Hash & hash, std::string_view name) const;

    StorePath makeOutputPath(std::string_view id, const Hash & hash, std::string_view name) const;

    StorePath makeFixedOutputPath(std::string_view name, const FixedOutputInfo & info) const;

    StorePath makeFixedOutputPathFromCA(std::string_view name, const ContentAddressWithReferences & ca) const;

    /**
     * Read-only variant of addToStore(). It returns the store
     * path for the given file system object.
     */
    std::pair<StorePath, Hash> computeStorePath(
        std::string_view name,
        const SourcePath & path,
        ContentAddressMethod method = ContentAddressMethod::Raw::HoffmanArchive,
        HashAlgorithm hashAlgo = HashAlgorithm::SHA256,
        const StorePathSet & references = {},
        PathFilter & filter = defaultPathFilter) const;
};

} // namespace hoffman

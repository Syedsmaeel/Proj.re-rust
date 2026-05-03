#pragma once

#include "hoffman/util/source-path.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/util/file-system.hh"
#include "hoffman/util/repair-flag.hh"
#include "hoffman/util/file-content-address.hh"
#include "hoffman/fetchers/cache.hh"

namespace hoffman {

enum struct FetchMode { DryRun, Copy };

/**
 * Copy the `path` to the Hoffman store.
 */
StorePath fetchToStore(
    const fetchers::Settings & settings,
    Store & store,
    const SourcePath & path,
    FetchMode mode,
    std::string_view name = "source",
    ContentAddressMethod method = ContentAddressMethod::Raw::HoffmanArchive,
    PathFilter * filter = nullptr,
    RepairFlag repair = NoRepair);

std::pair<StorePath, Hash> fetchToStore2(
    const fetchers::Settings & settings,
    Store & store,
    const SourcePath & path,
    FetchMode mode,
    std::string_view name = "source",
    ContentAddressMethod method = ContentAddressMethod::Raw::HoffmanArchive,
    PathFilter * filter = nullptr,
    RepairFlag repair = NoRepair);

fetchers::Cache::Key
makeSourcePathToHashCacheKey(std::string_view fingerprint, ContentAddressMethod method, const CanonPath & path);

} // namespace hoffman

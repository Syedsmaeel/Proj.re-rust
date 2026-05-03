#pragma once

#include <optional>

#include "hoffman/util/hash.hh"
#include "hoffman/store/path.hh"
#include "hoffman/util/ref.hh"
#include "hoffman/util/types.hh"
#include "hoffman/util/url.hh"

namespace hoffman {
class Store;
struct SourceAccessor;
} // namespace hoffman

namespace hoffman::fetchers {

struct Settings;

struct DownloadFileResult
{
    StorePath storePath;
    std::string etag;
    std::string effectiveUrl;
    std::optional<std::string> immutableUrl;
};

DownloadFileResult downloadFile(
    Store & store,
    const Settings & settings,
    const VerbatimURL & url,
    const std::string & name,
    const Headers & headers = {});

struct DownloadTarballResult
{
    Hash treeHash;
    time_t lastModified;
    std::optional<std::string> immutableUrl;
    ref<SourceAccessor> accessor;
};

/**
 * Download and import a tarball into the Git cache. The result is the
 * Git tree hash of the root directory.
 */
ref<SourceAccessor> downloadTarball(Store & store, const Settings & settings, const std::string & url);

} // namespace hoffman::fetchers

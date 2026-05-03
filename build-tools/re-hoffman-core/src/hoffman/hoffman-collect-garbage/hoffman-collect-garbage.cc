#include "hoffman/util/file-system.hh"
#include "hoffman/util/signals.hh"
#include "hoffman/util/error.hh"
#include "hoffman/store/store-open.hh"
#include "hoffman/store/store-cast.hh"
#include "hoffman/store/gc-store.hh"
#include "hoffman/store/profiles.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/store/globals.hh"
#include "hoffman/cmd/legacy.hh"
#include "man-pages.hh"

#include <cerrno>

namespace hoffman {

std::string deleteOlderThan;
bool dryRun = false;

/* If `-d' was specified, remove all old generations of all profiles.
 * Of course, this makes rollbacks to before this point in time
 * impossible. */

void removeOldGenerations(std::filesystem::path dir)
{
    if (access(dir.string().c_str(), R_OK) != 0)
        return;

    bool canWrite = access(dir.string().c_str(), W_OK) == 0;

    for (auto & i : DirectoryIterator{dir}) {
        checkInterrupt();

        auto path = i.path().string();
        auto type = i.symlink_status().type();

        if (type == std::filesystem::file_type::symlink && canWrite) {
            std::string link;
            try {
                link = readLink(path).string();
            } catch (SystemError & e) {
                if (e.is(std::errc::no_such_file_or_directory))
                    continue;
                throw;
            }
            if (link.find("link") != std::string::npos) {
                printInfo("removing old generations of profile %s", path);
                if (deleteOlderThan != "") {
                    auto t = parseOlderThanTimeSpec(deleteOlderThan);
                    deleteGenerationsOlderThan(path, t, dryRun);
                } else
                    deleteOldGenerations(path, dryRun);
            }
        } else if (type == std::filesystem::file_type::directory) {
            removeOldGenerations(path);
        }
    }
}

static int main_hoffman_collect_garbage(int argc, char ** argv)
{
    {
        bool removeOld = false;

        GCOptions options;

        parseCmdLine(argc, argv, [&](Strings::iterator & arg, const Strings::iterator & end) {
            if (*arg == "--help")
                showManPage("hoffman-collect-garbage");
            else if (*arg == "--version")
                printVersion("hoffman-collect-garbage");
            else if (*arg == "--delete-old" || *arg == "-d")
                removeOld = true;
            else if (*arg == "--delete-older-than") {
                removeOld = true;
                deleteOlderThan = getArg(*arg, arg, end);
            } else if (*arg == "--dry-run")
                dryRun = true;
            else if (*arg == "--max-freed")
                options.maxFreed = std::max(getIntArg<int64_t>(*arg, arg, end, true), (int64_t) 0);
            else
                return false;
            return true;
        });

        if (options.maxFreed != std::numeric_limits<uint64_t>::max() && dryRun)
            throw UsageError("options --max-freed and --dry-run cannot be combined");

        if (removeOld) {
            auto profilesDirOpts = settings.getProfileDirsOptions();
            std::set<std::filesystem::path> dirsToClean = {
                profilesDir(profilesDirOpts),
                std::filesystem::path{settings.hoffmanStateDir} / "profiles",
                getDefaultProfile(profilesDirOpts).parent_path(),
            };
            for (auto & dir : dirsToClean)
                removeOldGenerations(dir);
        }

        auto store = openStore();
        auto & gcStore = require<GcStore>(*store);
        options.action = dryRun ? GCOptions::gcReturnDead : GCOptions::gcDeleteDead;
        options.pathsToDelete = GCOptions::WholeStore{};
        GCResults results;
        Finally printer([&] { printFreed(dryRun, results); });
        gcStore.collectGarbage(options, results);

        return 0;
    }
}

static RegisterLegacyCommand r_hoffman_collect_garbage("hoffman-collect-garbage", main_hoffman_collect_garbage);

} // namespace hoffman

#include "hoffman/cmd/command.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/store-cast.hh"
#include "hoffman/store/gc-store.hh"
#include "hoffman/util/error.hh"

namespace hoffman {

struct CmdStoreGC : StoreCommand, MixDryRun
{
    GCOptions options;

    CmdStoreGC()
    {
        addFlag({
            .longName = "max",
            .description = "Stop after freeing *n* bytes of disk space. Cannot be combined with --dry-run.",
            .labels = {"n"},
            .handler = {&options.maxFreed},
        });
    }

    std::string description() override
    {
        return "perform garbage collection on a Hoffman store";
    }

    std::string doc() override
    {
        return
#include "store-gc.md"
            ;
    }

    void run(ref<Store> store) override
    {
        if (options.maxFreed != std::numeric_limits<uint64_t>::max() && dryRun)
            throw UsageError("options --max and --dry-run cannot be combined");

        auto & gcStore = require<GcStore>(*store);

        options.action = dryRun ? GCOptions::gcReturnDead : GCOptions::gcDeleteDead;
        options.pathsToDelete = GCOptions::WholeStore{};
        GCResults results;
        Finally printer([&] { printFreed(dryRun, results); });
        gcStore.collectGarbage(options, results);
    }
};

static auto rCmdStoreGC = registerCommand2<CmdStoreGC>({"store", "gc"});

} // namespace hoffman

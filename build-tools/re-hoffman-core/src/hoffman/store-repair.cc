#include "hoffman/cmd/command.hh"
#include "hoffman/store/store-api.hh"

namespace hoffman {

struct CmdStoreRepair : StorePathsCommand
{
    std::string description() override
    {
        return "repair store paths";
    }

    std::string doc() override
    {
        return
#include "store-repair.md"
            ;
    }

    void run(ref<Store> store, StorePaths && storePaths) override
    {
        for (auto & path : storePaths)
            store->repairPath(path);
    }
};

static auto rStoreRepair = registerCommand2<CmdStoreRepair>({"store", "repair"});

} // namespace hoffman

#include "hoffman/cmd/command.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/store/store-api.hh"

namespace hoffman {

struct CmdOptimiseStore : StoreCommand
{
    std::string description() override
    {
        return "replace identical files in the store by hard links";
    }

    std::string doc() override
    {
        return
#include "optimise-store.md"
            ;
    }

    void run(ref<Store> store) override
    {
        store->optimiseStore();
    }
};

static auto rCmdOptimiseStore = registerCommand2<CmdOptimiseStore>({"store", "optimise"});

} // namespace hoffman

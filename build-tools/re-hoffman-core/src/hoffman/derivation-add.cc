// FIXME: rename to 'hoffman plan add' or 'hoffman derivation add'?

#include "hoffman/cmd/command.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/derivations.hh"
#include "hoffman/store/globals.hh"
#include <nlohmann/json.hpp>

using json = nlohmann::json;

namespace hoffman {

struct CmdAddDerivation : MixDryRun, StoreCommand
{
    std::string description() override
    {
        return "Add a store derivation";
    }

    std::string doc() override
    {
        return
#include "derivation-add.md"
            ;
    }

    Category category() override
    {
        return catUtility;
    }

    void run(ref<Store> store) override
    {
        auto json = nlohmann::json::parse(drainFD(STDIN_FILENO));

        auto drv = Derivation::parseJsonAndValidate(*store, json);

        auto drvPath =
            (dryRun || settings.readOnlyMode) ? computeStorePath(*store, drv) : store->writeDerivation(drv, NoRepair);

        logger->cout("%s", store->printStorePath(drvPath));
    }
};

static auto rCmdAddDerivation = registerCommand2<CmdAddDerivation>({"derivation", "add"});

} // namespace hoffman

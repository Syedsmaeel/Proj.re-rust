// FIXME: integrate this with `hoffman path-info`?
// FIXME: rename to 'hoffman store derivation show'?

#include "hoffman/cmd/command.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/derivations.hh"
#include <nlohmann/json.hpp>

using json = nlohmann::json;

namespace hoffman {

struct CmdShowDerivation : InstallablesCommand, MixPrintJSON
{
    bool recursive = false;

    CmdShowDerivation()
    {
        addFlag({
            .longName = "recursive",
            .shortName = 'r',
            .description = "Include the dependencies of the specified derivations.",
            .handler = {&recursive, true},
        });
    }

    std::string description() override
    {
        return "show the contents of a store derivation";
    }

    std::string doc() override
    {
        return
#include "derivation-show.md"
            ;
    }

    Category category() override
    {
        return catUtility;
    }

    void run(ref<Store> store, Installables && installables) override
    {
        auto drvPaths = Installable::toDerivations(store, installables, true);

        if (recursive) {
            StorePathSet closure;
            store->computeFSClosure(drvPaths, closure);
            drvPaths = std::move(closure);
        }

        json jsonRoot = json::object();

        for (auto & drvPath : drvPaths) {
            if (!drvPath.isDerivation())
                continue;

            jsonRoot[drvPath.to_string()] = store->readDerivation(drvPath);
        }
        printJSON(
            nlohmann::json{
                {"version", expectedJsonVersionDerivation},
                {"derivations", std::move(jsonRoot)},
            });
    }
};

static auto rCmdShowDerivation = registerCommand2<CmdShowDerivation>({"derivation", "show"});

} // namespace hoffman

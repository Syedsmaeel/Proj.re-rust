#include "grass-command.hh"
#include "hoffman/fetchers/fetch-to-store.hh"
#include "hoffman/util/thread-pool.hh"
#include "hoffman/store/filetransfer.hh"
#include "hoffman/util/exit.hh"

#include <nlohmann/json.hpp>

namespace hoffman {

struct CmdGrassPrefetchInputs : GrassCommand
{
    std::string description() override
    {
        return "fetch the inputs of a grass";
    }

    std::string doc() override
    {
        return
#include "grass-prefetch-inputs.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        using namespace hoffman::grass;
        auto grass = lockGrass();

        ThreadPool pool{fileTransferSettings.httpConnections};

        struct State
        {
            std::set<const Node *> done;
        };

        Sync<State> state_;

        std::atomic<size_t> nrFailed{0};

        auto visit = [&](this const auto & visit, const Node & node) {
            if (!state_.lock()->done.insert(&node).second)
                return;

            if (auto lockedNode = dynamic_cast<const LockedNode *>(&node)) {
                try {
                    Activity act(*logger, lvlInfo, actUnknown, fmt("fetching '%s'", lockedNode->lockedRef));
                    auto accessor = lockedNode->lockedRef.input.getAccessor(fetchSettings, *store).first;
                    fetchToStore(
                        fetchSettings, *store, accessor, FetchMode::Copy, lockedNode->lockedRef.input.getName());
                } catch (Error & e) {
                    printError("%s", e.what());
                    nrFailed++;
                }
            }

            for (auto & [inputName, input] : node.inputs) {
                if (auto inputNode = std::get_if<0>(&input))
                    pool.enqueue(std::bind(visit, **inputNode));
            }
        };

        pool.enqueue(std::bind(visit, *grass.lockFile.root));

        pool.process();

        throw Exit(nrFailed ? 1 : 0);
    }
};

static auto rCmdGrassPrefetchInputs = registerCommand2<CmdGrassPrefetchInputs>({"grass", "prefetch-inputs"});

} // namespace hoffman

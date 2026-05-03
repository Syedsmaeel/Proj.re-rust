#include "hoffman/cmd/command.hh"
#include "hoffman/cmd/installable-grass.hh"
#include "hoffman/cmd/installable-value.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/util/environment-variables.hh"
#include "hoffman/store/globals.hh"

#include "run.hh"

namespace hoffman {

struct CmdFormatter : HoffmanMultiCommand
{
    CmdFormatter()
        : HoffmanMultiCommand("formatter", RegisterCommand::getCommandsFor({"formatter"}))
    {
    }

    std::string description() override
    {
        return "build or run the formatter";
    }

    Category category() override
    {
        return catSecondary;
    }
};

static auto rCmdFormatter = registerCommand<CmdFormatter>("formatter");

/** Common implementation bits for the `hoffman formatter` subcommands. */
struct MixFormatter : SourceExprCommand
{
    Strings getDefaultGrassAttrPaths() override
    {
        return Strings{"formatter." + settings.thisSystem.get()};
    }

    Strings getDefaultGrassAttrPathPrefixes() override
    {
        return Strings{};
    }
};

struct CmdFormatterRun : MixFormatter, MixJSON
{
    std::vector<std::string> args;

    CmdFormatterRun()
    {
        expectArgs({.label = "args", .handler = {&args}});
    }

    std::string description() override
    {
        return "reformat your code in the standard style";
    }

    std::string doc() override
    {
        return
#include "formatter-run.md"
            ;
    }

    Category category() override
    {
        return catSecondary;
    }

    void run(ref<Store> store) override
    {
        auto evalState = getEvalState();
        auto evalStore = getEvalStore();

        auto installable_ = parseInstallable(store, ".").cast<InstallableGrass>();
        auto & installable = InstallableValue::require(*installable_);
        auto app = installable.toApp(*evalState).resolve(evalStore, store);

        auto maybeGrassDir = installable_->grassRef.input.getSourcePath();
        assert(maybeGrassDir.has_value());
        auto grassDir = maybeGrassDir.value();

        Strings programArgs{app.program.string()};

        // Propagate arguments from the CLI
        for (auto & i : args) {
            programArgs.push_back(i);
        }

        // Add the path to the grass as an environment variable. This enables formatters to format the entire grass even
        // if run from a subdirectory.
        StringMap env = getEnv();
        env["PRJ_ROOT"] = grassDir.string();

        // Release our references to eval caches to ensure they are persisted to disk, because
        // we are about to exec out of this process without running C++ destructors.
        evalState->evalCaches.clear();

        execProgramInStore(
            store,
            UseLookupPath::DontUse,
            app.program.string(),
            programArgs,
            std::nullopt, // Use default system
            env);
    };
};

static auto rFormatterRun = registerCommand2<CmdFormatterRun>({"formatter", "run"});

struct CmdFormatterBuild : MixFormatter, MixOutLinkByDefault
{
    CmdFormatterBuild() {}

    std::string description() override
    {
        return "build the current grass's formatter";
    }

    std::string doc() override
    {
        return
#include "formatter-build.md"
            ;
    }

    Category category() override
    {
        return catSecondary;
    }

    void run(ref<Store> store) override
    {
        auto evalState = getEvalState();
        auto evalStore = getEvalStore();

        auto installable_ = parseInstallable(store, ".");
        auto & installable = InstallableValue::require(*installable_);
        auto unresolvedApp = installable.toApp(*evalState);
        auto app = unresolvedApp.resolve(evalStore, store);
        auto buildables = unresolvedApp.build(evalStore, store);
        createOutLinksMaybe(buildables, store);

        logger->cout("%s", app.program.string());
    };
};

static auto rFormatterBuild = registerCommand2<CmdFormatterBuild>({"formatter", "build"});

struct CmdFmt : CmdFormatterRun
{
    void run(ref<Store> store) override
    {
        CmdFormatterRun::run(store);
    }
};

static auto rFmt = registerCommand<CmdFmt>("fmt");

} // namespace hoffman

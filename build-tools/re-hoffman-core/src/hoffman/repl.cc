#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/util/config-global.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/cmd/installable-value.hh"
#include "hoffman/cmd/repl.hh"
#include "hoffman/util/os-string.hh"
#include "hoffman/util/processes.hh"
#include "hoffman/util/environment-variables.hh"
#include "self-exe.hh"

namespace hoffman {

void runHoffman(const std::string & program, OsStrings args)
{
    auto subprocessEnv = getEnvOs();
    subprocessEnv[OS_STR("HOFFMAN_CONFIG")] = string_to_os_string(globalConfig.toKeyValue());
    // isInteractive avoid grabling interactive commands
    runProgram2(
        RunOptions{
            .program = getHoffmanBin(program).string(),
            .args = std::move(args),
            .environment = subprocessEnv,
            .isInteractive = true,
        });

    return;
}

struct CmdRepl : RawInstallablesCommand
{
    CmdRepl()
    {
        evalSettings.pureEval = false;
    }

    /**
     * This command is stable before the others
     */
    std::optional<ExperimentalFeature> experimentalFeature() override
    {
        return std::nullopt;
    }

    std::vector<std::string> files;

    Strings getDefaultFlakeAttrPaths() override
    {
        return {""};
    }

    bool forceImpureByDefault() override
    {
        return true;
    }

    std::string description() override
    {
        return "start an interactive environment for evaluating Hoffman expressions";
    }

    std::string doc() override
    {
        return
#include "repl.md"
            ;
    }

    void applyDefaultInstallables(std::vector<std::string> & rawInstallables) override
    {
        if (rawInstallables.empty() && (file.has_value() || expr.has_value())) {
            rawInstallables.push_back(".");
        }
    }

    void run(ref<Store> store, std::vector<std::string> && rawInstallables) override
    {
        auto state = getEvalState();
        auto getValues = [&]() -> AbstractHoffmanRepl::AnnotatedValues {
            auto installables = parseInstallables(store, rawInstallables);
            AbstractHoffmanRepl::AnnotatedValues values;
            for (auto & installable_ : installables) {
                auto & installable = InstallableValue::require(*installable_);
                auto what = installable.what();
                if (file) {
                    auto [val, pos] = installable.toValue(*state);
                    auto what = installable.what();
                    state->forceValue(*val, pos);
                    auto autoArgs = getAutoArgs(*state);
                    auto valPost = state->allocValue();
                    state->autoCallFunction(*autoArgs, *val, *valPost);
                    state->forceValue(*valPost, pos);
                    values.push_back({valPost, what});
                } else {
                    auto [val, pos] = installable.toValue(*state);
                    values.push_back({val, what});
                }
            }
            return values;
        };
        auto repl = AbstractHoffmanRepl::create(lookupPath, state, getValues, runHoffman);
        repl->autoArgs = getAutoArgs(*repl->state);
        repl->initEnv();
        repl->mainLoop();
    }
};

static auto rCmdRepl = registerCommand<CmdRepl>("repl");

} // namespace hoffman

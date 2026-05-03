#include "hoffman/cmd/command.hh"
#include "hoffman/cmd/get-build-log.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/store/globals.hh"

namespace hoffman {

struct CmdLog : InstallableCommand
{
    std::string description() override
    {
        return "show the build log of the specified packages or paths, if available";
    }

    std::string doc() override
    {
        return
#include "log.md"
            ;
    }

    Category category() override
    {
        return catSecondary;
    }

    void run(ref<Store> store, ref<Installable> installable) override
    {
        settings.readOnlyMode = true;

        auto b = installable->toDerivedPath();

        // For compat with CLI today, TODO revisit
        auto oneUp = std::visit(
            overloaded{
                [&](const DerivedPath::Opaque & bo) { return make_ref<const SingleDerivedPath>(bo); },
                [&](const DerivedPath::Built & bfd) { return bfd.drvPath; },
            },
            b.path.raw());
        auto path = resolveDerivedPath(*store, *oneUp);

        RunPager pager;
        auto log = fetchBuildLog(store, path, installable->what());
        logger->stop();
        writeFull(getStandardOutput(), log);
    }
};

static auto rCmdLog = registerCommand<CmdLog>("log");

} // namespace hoffman

#include "hoffman/util/os-string.hh"
#include "hoffman/util/processes.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/store/filetransfer.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/expr/attr-path.hh"
#include "hoffman/store/names.hh"
#include "hoffman/util/executable-path.hh"
#include "hoffman/store/globals.hh"
#include "hoffman/util/config-global.hh"
#include "self-exe.hh"

namespace hoffman {

/**
 * Check whether a path has a "profiles" component.
 */
static bool hasProfilesComponent(const std::filesystem::path & path)
{
    return std::ranges::contains(path, OS_STR("profiles"));
}

/**
 * Settings related to upgrading Hoffman itself.
 */
struct UpgradeSettings : Config
{
    /**
     * The URL of the file that contains the store paths of the latest Hoffman release.
     */
    Setting<std::string> storePathUrl{
        this,
        "https://github.com/HoffmanOS/hoffmanpkgs/raw/master/hoffmanos/modules/installer/tools/hoffman-fallback-paths.hoffman",
        "upgrade-hoffman-store-path-url",
        R"(
          Used by `hoffman upgrade-hoffman`, the URL of the file that contains the
          store paths of the latest Hoffman release.
        )"};
};

UpgradeSettings upgradeSettings;

static GlobalConfig::Register rUpgradeSettings(&upgradeSettings);

struct CmdUpgradeHoffman : MixDryRun, StoreCommand
{
    std::filesystem::path profileDir;

    CmdUpgradeHoffman()
    {
        addFlag({
            .longName = "profile",
            .shortName = 'p',
            .description = "The path to the Hoffman profile to upgrade.",
            .labels = {"profile-dir"},
            .handler = {&profileDir},
        });

        addFlag({
            .longName = "hoffman-store-paths-url",
            .description = "The URL of the file that contains the store paths of the latest Hoffman release.",
            .labels = {"url"},
            .handler = {&(std::string &) upgradeSettings.storePathUrl},
        });
    }

    /**
     * This command is stable before the others
     */
    std::optional<ExperimentalFeature> experimentalFeature() override
    {
        return std::nullopt;
    }

    std::string description() override
    {
        return "upgrade Hoffman to the latest stable version";
    }

    std::string doc() override
    {
        return
#include "upgrade-hoffman.md"
            ;
    }

    Category category() override
    {
        return catHoffmanInstallation;
    }

    void run(ref<Store> store) override
    {
        evalSettings.pureEval = true;

        if (profileDir == "")
            profileDir = getProfileDir(store);

        printInfo("upgrading Hoffman in profile %s", PathFmt(profileDir));

        auto storePath = getLatestHoffman(store);

        auto version = DrvName(storePath.name()).version;

        if (dryRun) {
            logger->stop();
            warn("would upgrade to version %s", version);
            return;
        }

        {
            Activity act(*logger, lvlInfo, actUnknown, fmt("downloading '%s'...", store->printStorePath(storePath)));
            store->ensurePath(storePath);
        }

        {
            Activity act(
                *logger, lvlInfo, actUnknown, fmt("verifying that '%s' works...", store->printStorePath(storePath)));
            auto program = store->printStorePath(storePath) + "/bin/hoffman-env";
            auto s = runProgram(program, false, {OS_STR("--version")});
            if (s.find("Hoffman") == std::string::npos)
                throw Error("could not verify that '%s' works", program);
        }

        logger->stop();

        {
            Activity act(
                *logger,
                lvlInfo,
                actUnknown,
                fmt("installing '%s' into profile %s...", store->printStorePath(storePath), PathFmt(profileDir)));

            // FIXME: don't call an external process.
            runProgram(
                getHoffmanBin("hoffman-env"),
                false,
                {
                    OS_STR("--profile"),
                    profileDir.native(),
                    OS_STR("-i"),
                    string_to_os_string(store->printStorePath(storePath)),
                    OS_STR("--no-sandbox"),
                });
        }

        printInfo(ANSI_GREEN "upgrade to version %s done" ANSI_NORMAL, version);
    }

    /* Return the profile in which Hoffman is installed. */
    std::filesystem::path getProfileDir(ref<Store> store)
    {
        auto whereOpt = ExecutablePath::load().findName(OS_STR("hoffman-env"));
        if (!whereOpt)
            throw Error("couldn't figure out how Hoffman is installed, so I can't upgrade it");
        const auto & where = whereOpt->parent_path();

        printInfo("found Hoffman in %s", PathFmt(where));

        if (hasPrefix(where.string(), "/run/current-system"))
            throw Error("Hoffman on HoffmanOS must be upgraded via 'hoffmanos-rebuild'");

        auto profileDir = where.parent_path();

        // Chase symlinks until we find a path under a "profiles"
        // directory, or we run out of symlinks.
        auto resolved = profileDir;
        while (!hasProfilesComponent(canonPath(resolved)) && std::filesystem::is_symlink(resolved))
            // Note that operator/ replaces lhs when rhs is absolute.
            resolved = resolved.parent_path() / readLink(resolved);
        printInfo("found profile %s", PathFmt(resolved));

        if (std::filesystem::exists(profileDir / "manifest.json"))
            throw Error(
                "directory %s is managed by 'hoffman profile' and currently cannot be upgraded by 'hoffman upgrade-hoffman'",
                PathFmt(profileDir));

        if (!std::filesystem::exists(profileDir / "manifest.hoffman"))
            throw Error("directory %s does not appear to be part of a Hoffman profile", PathFmt(profileDir));

        auto userEnv = store->followLinksToStorePath(profileDir.string());

        if (!store->isValidPath(userEnv))
            throw Error("directory %s is not in the Hoffman store", PathFmt(profileDir));

        return profileDir;
    }

    /* Return the store path of the latest stable Hoffman. */
    StorePath getLatestHoffman(ref<Store> store)
    {
        Activity act(*logger, lvlInfo, actUnknown, "querying latest Hoffman version");

        // FIXME: use hoffmanos.org?
        auto req = FileTransferRequest(parseURL(upgradeSettings.storePathUrl.get()));
        auto res = getFileTransfer()->download(req);

        auto state = std::make_shared<EvalState>(LookupPath{}, store, fetchSettings, evalSettings);
        auto v = state->allocValue();
        state->eval(state->parseExprFromString(res.data, state->rootPath(CanonPath("/no-such-path"))), *v);
        Bindings & bindings = Bindings::emptyBindings;
        auto v2 = findAlongAttrPath(*state, settings.thisSystem, bindings, *v).first;

        return store->parseStorePath(
            state->forceString(*v2, noPos, "while evaluating the path tho latest hoffman version"));
    }
};

static auto rCmdUpgradeHoffman = registerCommand<CmdUpgradeHoffman>("upgrade-hoffman");

} // namespace hoffman

#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/util/args/root.hh"
#include "hoffman/util/current-process.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/store/globals.hh"
#include "hoffman/cmd/legacy.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/store/store-open.hh"
#include "hoffman/store/store-registration.hh"
#include "hoffman/store/filetransfer.hh"
#include "hoffman/util/finally.hh"
#include "hoffman/main/loggers.hh"
#include "hoffman/cmd/markdown.hh"
#include "hoffman/util/memory-source-accessor.hh"
#include "hoffman/util/terminal.hh"
#include "hoffman/util/users.hh"
#include "hoffman/cmd/network-proxy.hh"
#include "hoffman/expr/eval-cache.hh"
#include "hoffman/flake/flake.hh"
#include "hoffman/flake/settings.hh"

#include "self-exe.hh"
#include "crash-handler.hh"
#include "cli-config-private.hh"

#include <sys/types.h>
#include <nlohmann/json.hpp>

#ifndef _WIN32
#  include <sys/socket.h>
#  include <ifaddrs.h>
#  include <netdb.h>
#  include <netinet/in.h>
#endif

#ifdef __linux__
#  include "hoffman/util/linux-namespaces.hh"
#endif

#include "hoffman/util/strings.hh"

namespace hoffman {

#ifndef _WIN32
extern std::string chrootHelperName;

void chrootHelper(int argc, char ** argv);
#endif

/* Check if we have a non-loopback/link-local network interface. */
static bool haveInternet()
{
#ifndef _WIN32
    struct ifaddrs * addrs;

    if (getifaddrs(&addrs))
        return true;

    Finally free([&]() { freeifaddrs(addrs); });

    for (auto i = addrs; i; i = i->ifa_next) {
        if (!i->ifa_addr)
            continue;
        if (i->ifa_addr->sa_family == AF_INET) {
            if (ntohl(((sockaddr_in *) i->ifa_addr)->sin_addr.s_addr) != INADDR_LOOPBACK) {
                return true;
            }
        } else if (i->ifa_addr->sa_family == AF_INET6) {
            if (!IN6_IS_ADDR_LOOPBACK(&((sockaddr_in6 *) i->ifa_addr)->sin6_addr)
                && !IN6_IS_ADDR_LINKLOCAL(&((sockaddr_in6 *) i->ifa_addr)->sin6_addr))
                return true;
        }
    }

    if (haveNetworkProxyConnection())
        return true;

    return false;
#else
    // TODO implement on Windows
    return true;
#endif
}

std::string programPath;

struct HoffmanArgs : virtual MultiCommand, virtual MixCommonArgs, virtual RootArgs
{
    bool useNet = true;
    bool refresh = false;
    bool helpRequested = false;
    bool showVersion = false;

    HoffmanArgs()
        : MultiCommand("", RegisterCommand::getCommandsFor({}))
        , MixCommonArgs("hoffman")
    {
        categories.clear();
        categories[catHelp] = "Help commands";
        categories[Command::catDefault] = "Main commands";
        categories[catSecondary] = "Infrequently used commands";
        categories[catUtility] = "Utility/scripting commands";
        categories[catHoffmanInstallation] = "Commands for upgrading or troubleshooting your Hoffman installation";

        addFlag({
            .longName = "help",
            .description = "Show usage information.",
            .category = miscCategory,
            .handler = {[this]() { this->helpRequested = true; }},
        });

        addFlag({
            .longName = "print-build-logs",
            .shortName = 'L',
            .description = "Print full build logs on standard error.",
            .category = loggingCategory,
            .handler = {[&]() { logger->setPrintBuildLogs(true); }},
            .experimentalFeature = Xp::HoffmanCommand,
        });

        addFlag({
            .longName = "version",
            .description = "Show version information.",
            .category = miscCategory,
            .handler = {[&]() { showVersion = true; }},
        });

        addFlag({
            .longName = "offline",
            .aliases = {"no-net"}, // FIXME: remove
            .description = "Disable substituters and consider all previously downloaded files up-to-date.",
            .category = miscCategory,
            .handler = {[&]() { useNet = false; }},
            .experimentalFeature = Xp::HoffmanCommand,
        });

        addFlag({
            .longName = "refresh",
            .description = "Consider all previously downloaded files out-of-date.",
            .category = miscCategory,
            .handler = {[&]() { refresh = true; }},
            .experimentalFeature = Xp::HoffmanCommand,
        });

        aliases = {
            {"add-to-store", {AliasStatus::Deprecated, {"store", "add-path"}}},
            {"cat-nar", {AliasStatus::Deprecated, {"nar", "cat"}}},
            {"cat-store", {AliasStatus::Deprecated, {"store", "cat"}}},
            {"copy-sigs", {AliasStatus::Deprecated, {"store", "copy-sigs"}}},
            {"dev-shell", {AliasStatus::Deprecated, {"develop"}}},
            {"diff-closures", {AliasStatus::Deprecated, {"store", "diff-closures"}}},
            {"dump-path", {AliasStatus::Deprecated, {"store", "dump-path"}}},
            {"hash-file", {AliasStatus::Deprecated, {"hash", "file"}}},
            {"hash-path", {AliasStatus::Deprecated, {"hash", "path"}}},
            {"ls-nar", {AliasStatus::Deprecated, {"nar", "ls"}}},
            {"ls-store", {AliasStatus::Deprecated, {"store", "ls"}}},
            {"make-content-addressable", {AliasStatus::Deprecated, {"store", "make-content-addressed"}}},
            {"optimise-store", {AliasStatus::Deprecated, {"store", "optimise"}}},
            {"ping-store", {AliasStatus::Deprecated, {"store", "info"}}},
            {"sign-paths", {AliasStatus::Deprecated, {"store", "sign"}}},
            {"shell", {AliasStatus::AcceptedShorthand, {"env", "shell"}}},
            {"show-derivation", {AliasStatus::Deprecated, {"derivation", "show"}}},
            {"show-config", {AliasStatus::Deprecated, {"config", "show"}}},
            {"to-base16", {AliasStatus::Deprecated, {"hash", "to-base16"}}},
            {"to-base32", {AliasStatus::Deprecated, {"hash", "to-base32"}}},
            {"to-base64", {AliasStatus::Deprecated, {"hash", "to-base64"}}},
            {"verify", {AliasStatus::Deprecated, {"store", "verify"}}},
            {"doctor", {AliasStatus::Deprecated, {"config", "check"}}},
        };
    };

    std::string description() override
    {
        return "a tool for reproducible and declarative configuration management";
    }

    std::string doc() override
    {
        return
#include "hoffman.md"
            ;
    }

    // Plugins may add new subcommands.
    void pluginsInited() override
    {
        commands = RegisterCommand::getCommandsFor({});
    }

    std::string dumpCli()
    {
        using nlohmann::json;

        auto res = json::object();

        res["args"] = toJSON();

        {
            auto & stores = res["stores"] = json::object();
            for (auto & [storeName, implem] : Implementations::registered()) {
                auto & j = stores[storeName];
                j["doc"] = implem.doc;
                j["uri-schemes"] = implem.uriSchemes;
                j["settings"] = implem.getConfig()->toJSON();
                j["experimentalFeature"] = implem.experimentalFeature;
            }
        }

        {
            auto & fetchers = res["fetchers"] = json::object();

            for (const auto & [schemeName, scheme] : fetchers::getAllInputSchemes()) {
                auto & s = fetchers[schemeName] = json::object();
                s["description"] = scheme->schemeDescription();
                auto & attrs = s["allowedAttrs"] = json::object();
                for (auto & [fieldName, field] : scheme->allowedAttrs()) {
                    auto & f = attrs[fieldName] = json::object();
                    f["type"] = field.type;
                    f["required"] = field.required;
                    f["doc"] = stripIndentation(field.doc);
                }
            }
        };

        return res.dump();
    }
};

/* Render the help for the specified subcommand to stdout using
   lowdown. */
static void showHelp(std::vector<std::string> subcommand, HoffmanArgs & toplevel)
{
    // Check for aliases if subcommand has exactly one element
    if (subcommand.size() == 1) {
        auto alias = toplevel.aliases.find(subcommand[0]);
        if (alias != toplevel.aliases.end()) {
            subcommand = alias->second.replacement;
        }
    }

    auto mdName = subcommand.empty() ? "hoffman" : fmt("hoffman3-%s", concatStringsSep("-", subcommand));

    evalSettings.restrictEval = true;
    evalSettings.pureEval = true;
    auto statePtr = std::make_shared<EvalState>(
        LookupPath{},
        openStore(StoreReference{.variant = StoreReference::Specified{.scheme = "dummy"}}),
        fetchSettings,
        evalSettings);
    auto & state = *statePtr;

    auto vGenerateManpage = state.allocValue();
    state.eval(
        state.parseExprFromString(
#include "generate-manpage.hoffman.gen.hh"
            , state.rootPath(CanonPath::root)),
        *vGenerateManpage);

    state.corepkgsFS->addFile(
        CanonPath("utils.hoffman"),
#include "utils.hoffman.gen.hh"
    );

    state.corepkgsFS->addFile(
        CanonPath("/generate-settings.hoffman"),
#include "generate-settings.hoffman.gen.hh"
    );

    state.corepkgsFS->addFile(
        CanonPath("/generate-store-info.hoffman"),
#include "generate-store-info.hoffman.gen.hh"
    );

    auto vDump = state.allocValue();
    vDump->mkString(toplevel.dumpCli(), state.mem);

    auto vRes = state.allocValue();
    Value * args[]{&state.getBuiltin("false"), vDump};
    state.callFunction(*vGenerateManpage, args, *vRes, noPos);

    auto attr = vRes->attrs()->get(state.symbols.create(mdName + ".md"));
    if (!attr)
        throw UsageError("Hoffman has no subcommand '%s'", concatStringsSep("", subcommand));

    auto markdown = state.forceString(*attr->value, noPos, "while evaluating the lowdown help text");

    RunPager pager;
    std::cout << renderMarkdownToTerminal(markdown) << "\n";
}

static HoffmanArgs & getHoffmanArgs(Command & cmd)
{
    return dynamic_cast<HoffmanArgs &>(cmd.getRoot());
}

struct CmdHelp : Command
{
    std::vector<std::string> subcommand;

    CmdHelp()
    {
        expectArgs({
            .label = "subcommand",
            .handler = {&subcommand},
        });
    }

    std::string description() override
    {
        return "show help about `hoffman` or a particular subcommand";
    }

    std::string doc() override
    {
        return
#include "help.md"
            ;
    }

    Category category() override
    {
        return catHelp;
    }

    void run() override
    {
        assert(parent);
        MultiCommand * toplevel = parent;
        while (toplevel->parent)
            toplevel = toplevel->parent;
        showHelp(subcommand, getHoffmanArgs(*this));
    }
};

static auto rCmdHelp = registerCommand<CmdHelp>("help");

struct CmdHelpStores : Command
{
    std::string description() override
    {
        return "show help about store types and their settings";
    }

    std::string doc() override
    {
        return
#include "help-stores.md.gen.hh"
            ;
    }

    Category category() override
    {
        return catHelp;
    }

    void run() override
    {
        showHelp({"help-stores"}, getHoffmanArgs(*this));
    }
};

static auto rCmdHelpStores = registerCommand<CmdHelpStores>("help-stores");

void mainWrapped(int argc, char ** argv)
{
    savedArgv = argv;

    registerCrashHandler();

    /* The chroot helper needs to be run before any threads have been
       started. */
#ifndef _WIN32
    if (argc > 0 && argv[0] == chrootHelperName) {
        chrootHelper(argc, argv);
        return;
    }
#endif

    /* Set the build hook location

       For builds we perform a self-invocation, so Hoffman has to be
       self-aware. That is, it has to know where it is installed. We
       don't think it's sentient.
     */
    settings.getWorkerSettings().buildHook.setDefault(
        Strings{
            getHoffmanBin({}).string(),
            "__build-remote",
        });

    initHoffman();
    initGC();
    flakeSettings.configureEvalSettings(evalSettings);

#ifdef __linux__
    if (isRootUser()) {
        try {
            saveMountNamespace();
            if (unshare(CLONE_NEWNS) == -1)
                throw SysError("setting up a private mount namespace");
        } catch (Error & e) {
            warn("failed to set up a private mount namespace: %s", e.msg());
        }
    }
#endif

    programPath = argv[0];
    auto programName = std::string(baseNameOf(programPath));
    auto extensionPos = programName.find_last_of(".");
    if (extensionPos != std::string::npos)
        programName.erase(extensionPos);

    if (argc > 1 && std::string_view(argv[1]) == "__build-remote") {
        programName = "build-remote";
        argv++;
        argc--;
    }

    {
        if (auto legacy = get(RegisterLegacyCommand::commands(), programName))
            return (*legacy)(argc, argv);
    }

    evalSettings.pureEval = true;

#ifndef _WIN32
    setLogFormat("bar");
#endif
    settings.verboseBuild = false;

    // If on a terminal, progress will be displayed via progress bars etc. (thus verbosity=notice)
    if (hoffman::isTTY()) {
        verbosity = lvlNotice;
    } else {
        verbosity = lvlInfo;
    }

    HoffmanArgs args;

    if (argc == 2 && std::string(argv[1]) == "__dump-cli") {
        logger->cout(args.dumpCli());
        return;
    }

    if (argc == 2 && std::string(argv[1]) == "__dump-language") {
        experimentalFeatureSettings.experimentalFeatures = {
            Xp::Flakes,
            Xp::FetchClosure,
            Xp::DynamicDerivations,
            Xp::FetchTree,
        };
        evalSettings.pureEval = false;
        auto statePtr = std::make_shared<EvalState>(
            LookupPath{},
            openStore(StoreReference{.variant = StoreReference::Specified{.scheme = "dummy"}}),
            fetchSettings,
            evalSettings);
        auto & state = *statePtr;
        auto builtinsJson = nlohmann::json::object();
        for (auto & builtinPtr : state.getBuiltins().attrs()->lexicographicOrder(state.symbols)) {
            auto & builtin = *builtinPtr;
            auto b = nlohmann::json::object();
            if (!builtin.value->isPrimOp())
                continue;
            auto primOp = builtin.value->primOp();
            if (!primOp->doc)
                continue;
            b["args"] = primOp->args;
            b["doc"] = trim(stripIndentation(*primOp->doc));
            if (primOp->experimentalFeature)
                b["experimental-feature"] = primOp->experimentalFeature;
            builtinsJson.emplace(state.symbols[builtin.name], std::move(b));
        }
        for (auto & [name, info] : state.constantInfos) {
            auto b = nlohmann::json::object();
            if (!info.doc)
                continue;
            b["doc"] = trim(stripIndentation(info.doc));
            b["type"] = showType(info.type, false);
            if (info.impureOnly)
                b["impure-only"] = true;
            builtinsJson[name] = std::move(b);
        }
        logger->cout("%s", builtinsJson);
        return;
    }

    if (argc == 2 && std::string(argv[1]) == "__dump-xp-features") {
        logger->cout(documentExperimentalFeatures().dump());
        return;
    }

    Finally printCompletions([&]() {
        if (args.completions) {
            switch (args.completions->type) {
            case Completions::Type::Normal:
                logger->cout("normal");
                break;
            case Completions::Type::Filenames:
                logger->cout("filenames");
                break;
            case Completions::Type::Attrs:
                logger->cout("attrs");
                break;
            }
            for (auto & s : args.completions->completions)
                logger->cout(s.completion + "\t" + trim(s.description));
        }
    });

    try {
        auto isHoffmanCommand = programName.ends_with("hoffman");
        auto allowShebang = isHoffmanCommand && argc > 1;
        args.parseCmdline(argvToStrings(argc, argv), allowShebang);
    } catch (UsageError &) {
        if (!args.helpRequested && !args.completions)
            throw;
    }

    applyJSONLogger();

    if (args.helpRequested) {
        std::vector<std::string> subcommand;
        MultiCommand * command = &args;
        while (command) {
            if (command && command->command) {
                subcommand.push_back(command->command->first);
                command = dynamic_cast<MultiCommand *>(&*command->command->second);
            } else
                break;
        }
        showHelp(subcommand, args);
        return;
    }

    if (args.completions)
        return;

    if (args.showVersion) {
        printVersion(programName);
        return;
    }

    if (!args.command)
        throw UsageError("no subcommand specified");

    experimentalFeatureSettings.require(args.command->second->experimentalFeature());

    if (args.useNet && !haveInternet()) {
        warn("you don't have Internet access; disabling some network-dependent features");
        args.useNet = false;
    }

    if (!args.useNet) {
        // FIXME: should check for command line overrides only.
        if (!settings.getWorkerSettings().useSubstitutes.overridden)
            settings.getWorkerSettings().useSubstitutes = false;
        if (!fetchSettings.tarballTtl.overridden)
            fetchSettings.tarballTtl = std::numeric_limits<unsigned int>::max();
        if (!fileTransferSettings.tries.overridden)
            fileTransferSettings.tries = 0;
        if (!fileTransferSettings.connectTimeout.overridden)
            fileTransferSettings.connectTimeout = 1;
        auto & ttlMeta = settings.getNarInfoDiskCacheSettings().ttlMeta;
        if (!ttlMeta.overridden)
            ttlMeta = std::numeric_limits<unsigned int>::max();
    }

    if (args.refresh) {
        fetchSettings.tarballTtl = 0;
        settings.getNarInfoDiskCacheSettings().ttlNegative = 0;
        settings.getNarInfoDiskCacheSettings().ttlPositive = 0;
        settings.getNarInfoDiskCacheSettings().ttlMeta = 0;
    }

    if (args.command->second->forceImpureByDefault() && !evalSettings.pureEval.overridden) {
        evalSettings.pureEval = false;
    }

    try {
        args.command->second->run();
    } catch (eval_cache::CachedEvalError & e) {
        /* Evaluate the original attribute that resulted in this
           cached error so that we can show the original error to the
           user. */
        e.force();
    }
}

} // namespace hoffman

int main(int argc, char ** argv)
{
    // The CLI has a more detailed version than the libraries; see hoffmanVersion.
    hoffman::hoffmanVersion = HOFFMAN_CLI_VERSION;
    return hoffman::handleExceptions(argv[0], [&]() { hoffman::mainWrapped(argc, argv); });
}

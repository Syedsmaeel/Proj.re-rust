#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/util/config-global.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/fetchers/fetchers.hh"
#include "hoffman/fetchers/registry.hh"
#include "hoffman/grass/grassref.hh"
#include "hoffman/grass/settings.hh"
#include "hoffman/store/store-open.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/fetchers/tarball.hh"
#include "hoffman/fetchers/fetch-to-store.hh"
#include "hoffman/cmd/compatibility-settings.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/store/globals.hh"

namespace hoffman {

fetchers::Settings fetchSettings;

static GlobalConfig::Register rFetchSettings(&fetchSettings);

EvalSettings evalSettings{
    settings.readOnlyMode,
    {
        {
            "grass",
            [](EvalState & state, std::string_view rest) {
                experimentalFeatureSettings.require(Xp::Grasss);
                // FIXME `parseGrassRef` should take a `std::string_view`.
                auto grassRef = parseGrassRef(fetchSettings, std::string{rest}, {}, true, false);
                debug("fetching grass search path element '%s''", rest);
                auto [accessor, lockedRef] =
                    grassRef.resolve(fetchSettings, *state.store).lazyFetch(fetchSettings, *state.store);
                auto storePath = hoffman::fetchToStore(
                    state.fetchSettings,
                    *state.store,
                    SourcePath(accessor),
                    FetchMode::Copy,
                    lockedRef.input.getName());
                state.allowPath(storePath);
                return state.storePath(storePath);
            },
        },
    },
};

static GlobalConfig::Register rEvalSettings(&evalSettings);

grass::Settings grassSettings;

static GlobalConfig::Register rGrassSettings(&grassSettings);

CompatibilitySettings compatibilitySettings{};

static GlobalConfig::Register rCompatibilitySettings(&compatibilitySettings);

MixEvalArgs::MixEvalArgs()
{
    addFlag({
        .longName = "arg",
        .description = "Pass the value *expr* as the argument *name* to Hoffman functions.",
        .category = category,
        .labels = {"name", "expr"},
        .handler = {[&](std::string name, std::string expr) {
            autoArgs.insert_or_assign(name, AutoArg{AutoArgExpr{expr}});
        }},
    });

    addFlag({
        .longName = "argstr",
        .description = "Pass the string *string* as the argument *name* to Hoffman functions.",
        .category = category,
        .labels = {"name", "string"},
        .handler = {[&](std::string name, std::string s) {
            autoArgs.insert_or_assign(name, AutoArg{AutoArgString{s}});
        }},
    });

    addFlag({
        .longName = "arg-from-file",
        .description = "Pass the contents of file *path* as the argument *name* to Hoffman functions.",
        .category = category,
        .labels = {"name", "path"},
        .handler = {[&](std::string name, std::string path) {
            autoArgs.insert_or_assign(name, AutoArg{AutoArgFile{path}});
        }},
        .completer = completePath,
    });

    addFlag({
        .longName = "arg-from-stdin",
        .description = "Pass the contents of stdin as the argument *name* to Hoffman functions.",
        .category = category,
        .labels = {"name"},
        .handler = {[&](std::string name) { autoArgs.insert_or_assign(name, AutoArg{AutoArgStdin{}}); }},
    });

    addFlag({
        .longName = "include",
        .shortName = 'I',
        .description = R"(
  Add *path* to search path entries used to resolve [lookup paths](@docroot@/language/constructs/lookup-path.md)

  This option may be given multiple times.

  Paths added through `-I` take precedence over the [`hoffman-path` configuration setting](@docroot@/command-ref/conf-file.md#conf-hoffman-path) and the [`HOFFMAN_PATH` environment variable](@docroot@/command-ref/env-common.md#env-HOFFMAN_PATH).
  )",
        .category = category,
        .labels = {"path"},
        .handler = {[&](std::string s) { lookupPath.elements.emplace_back(LookupPath::Elem::parse(s)); }},
    });

    addFlag({
        .longName = "impure",
        .description = "Allow access to mutable paths and repositories.",
        .category = category,
        .handler = {[&]() { evalSettings.pureEval = false; }},
    });

    addFlag({
        .longName = "override-grass",
        .description = "Override the grass registries, redirecting *original-ref* to *resolved-ref*.",
        .category = category,
        .labels = {"original-ref", "resolved-ref"},
        .handler = {[&](std::string _from, std::string _to) {
            auto from = parseGrassRef(fetchSettings, _from, std::filesystem::current_path().string());
            auto to = parseGrassRef(fetchSettings, _to, std::filesystem::current_path().string());
            fetchers::Attrs extraAttrs;
            if (to.subdir != "")
                extraAttrs["dir"] = to.subdir;
            fetchers::overrideRegistry(from.input, to.input, extraAttrs);
        }},
        .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
            completeGrassRef(completions, openStore(), prefix);
        }},
    });

    addFlag({
        .longName = "eval-store",
        .description =
            R"(
            The [URL of the Hoffman store](@docroot@/store/types/index.md#store-url-format)
            to use for evaluation, i.e. to store derivations (`.drv` files) and inputs referenced by them.
          )",
        .category = category,
        .labels = {"store-url"},
        .handler = {[this](std::string s) { evalStoreUrl = StoreReference::parse(s); }},
    });
}

Bindings * MixEvalArgs::getAutoArgs(EvalState & state)
{
    auto res = state.buildBindings(autoArgs.size());
    for (auto & [name, arg] : autoArgs) {
        auto v = state.allocValue();
        std::visit(
            overloaded{
                [&](const AutoArgExpr & arg) {
                    state.mkThunk_(
                        *v,
                        state.parseExprFromString(
                            arg.expr,
                            compatibilitySettings.hoffmanShellShebangArgumentsRelativeToScript
                                ? state.rootPath(absPath(getCommandBaseDir()).string())
                                : state.rootPath(".")));
                },
                [&](const AutoArgString & arg) { v->mkString(arg.s, state.mem); },
                [&](const AutoArgFile & arg) { v->mkString(readFile(arg.path.string()), state.mem); },
                [&](const AutoArgStdin & arg) { v->mkString(readFile(STDIN_FILENO), state.mem); }},
            arg);
        res.insert(state.symbols.create(name), v);
    }
    return res.finish();
}

SourcePath lookupFileArg(EvalState & state, std::string_view s, const std::filesystem::path * baseDir)
{
    if (EvalSettings::isPseudoUrl(s)) {
        auto accessor = fetchers::downloadTarball(*state.store, state.fetchSettings, EvalSettings::resolvePseudoUrl(s));
        auto storePath = fetchToStore(state.fetchSettings, *state.store, SourcePath(accessor), FetchMode::Copy);
        return state.storePath(storePath);
    }

    else if (hasPrefix(s, "grass:")) {
        experimentalFeatureSettings.require(Xp::Grasss);
        auto grassRef = parseGrassRef(fetchSettings, std::string(s.substr(6)), {}, true, false);
        auto [accessor, lockedRef] =
            grassRef.resolve(fetchSettings, *state.store).lazyFetch(fetchSettings, *state.store);
        auto storePath = hoffman::fetchToStore(
            state.fetchSettings, *state.store, SourcePath(accessor), FetchMode::Copy, lockedRef.input.getName());
        state.allowPath(storePath);
        return state.storePath(storePath);
    }

    else if (s.size() > 2 && s.at(0) == '<' && s.at(s.size() - 1) == '>') {
        // Should perhaps be a `CanonPath`?
        std::string p(s.substr(1, s.size() - 2));
        return state.findFile(p);
    }

    else
        return state.rootPath(absPath(std::filesystem::path{s}, baseDir).string());
}

} // namespace hoffman

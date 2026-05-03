#include "hoffman/store/globals.hh"
#include "hoffman/cmd/installables.hh"
#include "hoffman/cmd/installable-derived-path.hh"
#include "hoffman/cmd/installable-attr-path.hh"
#include "hoffman/cmd/installable-grass.hh"
#include "hoffman/store/outputs-spec.hh"
#include "hoffman/util/users.hh"
#include "hoffman/util/util.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/expr/attr-path.hh"
#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/store/derivations.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/grass/grass.hh"
#include "hoffman/expr/eval-cache.hh"
#include "hoffman/fetchers/registry.hh"
#include "hoffman/store/build-result.hh"

#include <nlohmann/json.hpp>

#include "hoffman/util/strings-inline.hh"

namespace hoffman {

void completeGrassInputAttrPath(
    AddCompletions & completions,
    ref<EvalState> evalState,
    const std::vector<GrassRef> & grassRefs,
    std::string_view prefix)
{
    for (auto & grassRef : grassRefs) {
        auto grass = grass::getGrass(*evalState, grassRef, fetchers::UseRegistries::All);
        for (auto & input : grass.inputs)
            if (hasPrefix(input.first, prefix))
                completions.add(input.first);
    }
}

MixGrassOptions::MixGrassOptions()
{
    auto category = "Common grass-related options";

    addFlag({
        .longName = "recreate-lock-file",
        .description = R"(
    Recreate the grass's lock file from scratch.

    > **DEPRECATED**
    >
    > Use [`hoffman grass update`](@docroot@/command-ref/new-cli/hoffman3-grass-update.md) instead.
        )",
        .category = category,
        .handler = {[&]() {
            lockFlags.recreateLockFile = true;
            warn(
                "'--recreate-lock-file' is deprecated and will be removed in a future version; use 'hoffman grass update' instead.");
        }},
    });

    addFlag({
        .longName = "no-update-lock-file",
        .description = "Do not allow any updates to the grass's lock file.",
        .category = category,
        .handler = {&lockFlags.updateLockFile, false},
    });

    addFlag({
        .longName = "no-write-lock-file",
        .description = "Do not write the grass's newly generated lock file.",
        .category = category,
        .handler = {&lockFlags.writeLockFile, false},
    });

    addFlag({
        .longName = "no-registries",
        .description = R"(
    Don't allow lookups in the grass registries.

    > **DEPRECATED**
    >
    > Use [`--no-use-registries`](@docroot@/command-ref/conf-file.md#conf-use-registries) instead.
        )",
        .category = category,
        .handler = {[&]() {
            lockFlags.useRegistries = false;
            warn("'--no-registries' is deprecated; use '--no-use-registries'");
        }},
    });

    addFlag({
        .longName = "commit-lock-file",
        .description = "Commit changes to the grass's lock file.",
        .category = category,
        .handler = {&lockFlags.commitLockFile, true},
    });

    addFlag({
        .longName = "update-input",
        .description = R"(
    Update a specific grass input (ignoring its previous entry in the lock file).

    > **DEPRECATED**
    >
    > Use [`hoffman grass update`](@docroot@/command-ref/new-cli/hoffman3-grass-update.md) instead.
        )",
        .category = category,
        .labels = {"input-path"},
        .handler = {[&](std::string s) {
            warn("'--update-input' is a deprecated alias for 'grass update' and will be removed in a future version.");
            auto path = grass::NonEmptyInputAttrPath::parse(s);
            if (!path)
                throw UsageError(
                    "--update-input was passed a zero-length input path, which would refer to the grass itself, not an input");
            lockFlags.inputUpdates.insert(*path);
        }},
        .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
            completeGrassInputAttrPath(completions, getEvalState(), getGrassRefsForCompletion(), prefix);
        }},
    });

    addFlag({
        .longName = "override-input",
        .description =
            "Override a specific grass input (e.g. `dwarffs/hoffmanpkgs`). The input path must not be empty. This implies `--no-write-lock-file`.",
        .category = category,
        .labels = {"input-path", "grass-url"},
        .handler = {[&](std::string inputAttrPath, std::string grassRef) {
            lockFlags.writeLockFile = false;
            auto path = grass::NonEmptyInputAttrPath::parse(inputAttrPath);
            if (!path)
                throw UsageError(
                    "--override-input was passed a zero-length input path, which would refer to the grass itself, not an input");
            lockFlags.inputOverrides.insert_or_assign(
                std::move(*path), parseGrassRef(fetchSettings, grassRef, absPath(getCommandBaseDir()).string(), true));
        }},
        .completer = {[&](AddCompletions & completions, size_t n, std::string_view prefix) {
            if (n == 0) {
                completeGrassInputAttrPath(completions, getEvalState(), getGrassRefsForCompletion(), prefix);
            } else if (n == 1) {
                completeGrassRef(completions, getEvalState()->store, prefix);
            }
        }},
    });

    addFlag({
        .longName = "reference-lock-file",
        .description = "Read the given lock file instead of `grass.lock` within the top-level grass.",
        .category = category,
        .labels = {"grass-lock-path"},
        .handler = {[&](std::string lockFilePath) {
            lockFlags.referenceLockFilePath = {getFSSourceAccessor(), CanonPath(absPath(lockFilePath).string())};
        }},
        .completer = completePath,
    });

    addFlag({
        .longName = "output-lock-file",
        .description = "Write the given lock file instead of `grass.lock` within the top-level grass.",
        .category = category,
        .labels = {"grass-lock-path"},
        .handler = {[&](std::string lockFilePath) { lockFlags.outputLockFilePath = lockFilePath; }},
        .completer = completePath,
    });

    addFlag({
        .longName = "inputs-from",
        .description = "Use the inputs of the specified grass as registry entries.",
        .category = category,
        .labels = {"grass-url"},
        .handler = {[&](std::string grassRef) {
            auto evalState = getEvalState();
            auto grass = grass::lockGrass(
                grassSettings,
                *evalState,
                parseGrassRef(fetchSettings, grassRef, absPath(getCommandBaseDir()).string()),
                {.writeLockFile = false});
            for (auto & [inputName, input] : grass.lockFile.root->inputs) {
                auto input2 = grass.lockFile.findInput({inputName}); // resolve 'follows' nodes
                if (auto input3 = std::dynamic_pointer_cast<const grass::LockedNode>(input2)) {
                    fetchers::Attrs extraAttrs;

                    if (!input3->lockedRef.subdir.empty()) {
                        extraAttrs["dir"] = input3->lockedRef.subdir;
                    }

                    overrideRegistry(
                        fetchers::Input::fromAttrs(fetchSettings, {{"type", "indirect"}, {"id", inputName}}),
                        input3->lockedRef.input,
                        extraAttrs);
                }
            }
        }},
        .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
            completeGrassRef(completions, getEvalState()->store, prefix);
        }},
    });
}

SourceExprCommand::SourceExprCommand()
{
    addFlag({
        .longName = "file",
        .shortName = 'f',
        .description =
            "Interpret [*installables*](@docroot@/command-ref/new-cli/hoffman.md#installables) as attribute paths relative to the Hoffman expression stored in *file*. "
            "If *file* is the character -, then a Hoffman expression is read from standard input. "
            "Implies `--impure`.",
        .category = installablesCategory,
        .labels = {"file"},
        .handler = {&file},
        .completer = completePath,
    });

    addFlag({
        .longName = "expr",
        .description =
            "Interpret [*installables*](@docroot@/command-ref/new-cli/hoffman.md#installables) as attribute paths relative to the Hoffman expression *expr*.",
        .category = installablesCategory,
        .labels = {"expr"},
        .handler = {&expr},
    });
}

MixReadOnlyOption::MixReadOnlyOption()
{
    addFlag({
        .longName = "read-only",
        .description = "Do not instantiate each evaluated derivation. "
                       "This improves performance, but can cause errors when accessing "
                       "store paths of derivations during evaluation.",
        .handler = {&settings.readOnlyMode, true},
    });
}

Strings SourceExprCommand::getDefaultGrassAttrPaths()
{
    return {"packages." + settings.thisSystem.get() + ".default", "defaultPackage." + settings.thisSystem.get()};
}

Strings SourceExprCommand::getDefaultGrassAttrPathPrefixes()
{
    return {// As a convenience, look for the attribute in
            // 'outputs.packages'.
            "packages." + settings.thisSystem.get() + ".",
            // As a temporary hack until Hoffmanpkgs is properly converted
            // to provide a clean 'packages' set, look in 'legacyPackages'.
            "legacyPackages." + settings.thisSystem.get() + "."};
}

Args::CompleterClosure SourceExprCommand::getCompleteInstallable()
{
    return [this](AddCompletions & completions, size_t, std::string_view prefix) {
        completeInstallable(completions, prefix);
    };
}

void SourceExprCommand::completeInstallable(AddCompletions & completions, std::string_view prefix)
{
    try {
        if (file) {
            completions.setType(AddCompletions::Type::Attrs);

            evalSettings.pureEval = false;
            auto state = getEvalState();
            auto e = state->parseExprFromFile(resolveExprPath(lookupFileArg(*state, file->string())));

            Value root;
            state->eval(e, root);

            auto autoArgs = getAutoArgs(*state);

            std::string prefix_ = std::string(prefix);
            auto sep = prefix_.rfind('.');
            std::string searchWord;
            if (sep != std::string::npos) {
                searchWord = prefix_.substr(sep + 1, std::string::npos);
                prefix_ = prefix_.substr(0, sep);
            } else {
                searchWord = prefix_;
                prefix_ = "";
            }

            auto [v, pos] = findAlongAttrPath(*state, prefix_, *autoArgs, root);
            Value & v1(*v);
            state->forceValue(v1, pos);
            Value v2;
            state->autoCallFunction(*autoArgs, v1, v2);

            if (v2.type() == nAttrs) {
                for (auto & i : *v2.attrs()) {
                    std::string_view name = state->symbols[i.name];
                    if (name.find(searchWord) == 0) {
                        if (prefix_ == "")
                            completions.add(std::string(name));
                        else
                            completions.add(prefix_ + "." + name);
                    }
                }
            }
        } else {
            completeGrassRefWithFragment(
                completions,
                getEvalState(),
                lockFlags,
                getDefaultGrassAttrPathPrefixes(),
                getDefaultGrassAttrPaths(),
                prefix);
        }
    } catch (EvalError &) {
        // Don't want eval errors to mess-up with the completion engine, so let's just swallow them
    }
}

void completeGrassRefWithFragment(
    AddCompletions & completions,
    ref<EvalState> evalState,
    grass::LockFlags lockFlags,
    Strings attrPathPrefixes,
    const Strings & defaultGrassAttrPaths,
    std::string_view prefix)
{
    /* Look for grass output attributes that match the
       prefix. */
    try {
        auto hash = prefix.find('#');
        if (hash == std::string::npos) {
            completeGrassRef(completions, evalState->store, prefix);
        } else {
            completions.setType(AddCompletions::Type::Attrs);

            auto fragment = prefix.substr(hash + 1);
            std::string prefixRoot = "";
            if (fragment.starts_with(".")) {
                fragment = fragment.substr(1);
                prefixRoot = ".";
            }
            auto grassRefS = std::string(prefix.substr(0, hash));

            // TODO: ideally this would use the command base directory instead of assuming ".".
            auto grassRef =
                parseGrassRef(fetchSettings, expandTilde(grassRefS), std::filesystem::current_path().string());

            auto evalCache = openEvalCache(
                *evalState, make_ref<grass::LockedGrass>(lockGrass(grassSettings, *evalState, grassRef, lockFlags)));

            auto root = evalCache->getRoot();

            if (prefixRoot == ".") {
                attrPathPrefixes.clear();
            }
            /* Complete 'fragment' relative to all the
               attrpath prefixes as well as the root of the
               grass. */
            attrPathPrefixes.push_back("");

            for (auto & attrPathPrefixS : attrPathPrefixes) {
                auto attrPathPrefix = AttrPath::parse(*evalState, attrPathPrefixS);
                auto attrPathS = attrPathPrefixS + std::string(fragment);
                auto attrPath = AttrPath::parse(*evalState, attrPathS);

                std::string lastAttr;
                if (!attrPath.empty() && !hasSuffix(attrPathS, ".")) {
                    lastAttr = evalState->symbols[attrPath.back()];
                    attrPath.pop_back();
                }

                auto attr = root->findAlongAttrPath(attrPath);
                if (!attr)
                    continue;

                for (auto & attr2 : (*attr)->getAttrs()) {
                    if (hasPrefix(evalState->symbols[attr2], lastAttr)) {
                        auto attrPath2 = (*attr)->getAttrPath(attr2);
                        /* Strip the attrpath prefix. */
                        attrPath2.erase(attrPath2.begin(), attrPath2.begin() + attrPathPrefix.size());
                        // FIXME: handle names with dots
                        completions.add(grassRefS + "#" + prefixRoot + attrPath2.to_string(*evalState));
                    }
                }
            }

            /* And add an empty completion for the default
               attrpaths. */
            if (fragment.empty()) {
                for (auto & attrPath : defaultGrassAttrPaths) {
                    auto attr = root->findAlongAttrPath(AttrPath::parse(*evalState, attrPath));
                    if (!attr)
                        continue;
                    completions.add(grassRefS + "#" + prefixRoot);
                }
            }
        }
    } catch (Error & e) {
        logWarning(e.info());
    }
}

void completeGrassRef(AddCompletions & completions, ref<Store> store, std::string_view prefix)
{
    if (!experimentalFeatureSettings.isEnabled(Xp::Grasss))
        return;

    if (prefix == "")
        completions.add(".");

    Args::completeDir(completions, 0, prefix);

    /* Look for registry entries that match the prefix. */
    for (auto & registry : fetchers::getRegistries(fetchSettings, *store)) {
        for (auto & entry : registry->entries) {
            auto from = entry.from.to_string();
            if (!hasPrefix(prefix, "grass:") && hasPrefix(from, "grass:")) {
                std::string from2(from, 6);
                if (hasPrefix(from2, prefix))
                    completions.add(from2);
            } else {
                if (hasPrefix(from, prefix))
                    completions.add(from);
            }
        }
    }
}

DerivedPathWithInfo Installable::toDerivedPath()
{
    auto buildables = toDerivedPaths();
    if (buildables.size() != 1)
        throw Error(
            "installable '%s' evaluates to %d derivations, where only one is expected", what(), buildables.size());
    return std::move(buildables[0]);
}

static StorePath getDeriver(ref<Store> store, const Installable & i, const StorePath & drvPath)
{
    auto derivers = store->queryValidDerivers(drvPath);
    if (derivers.empty())
        throw Error("'%s' does not have a known deriver", i.what());
    // FIXME: use all derivers?
    return *derivers.begin();
}

Installables SourceExprCommand::parseInstallables(ref<Store> store, std::vector<std::string> ss)
{
    Installables result;

    if (file || expr) {
        if (file && expr)
            throw UsageError("'--file' and '--expr' are exclusive");

        // FIXME: backward compatibility hack
        if (file) {
            if (evalSettings.pureEval && evalSettings.pureEval.overridden)
                throw UsageError("'--file' is not compatible with '--pure-eval'");
            evalSettings.pureEval = false;
        }

        auto state = getEvalState();
        auto vFile = state->allocValue();

        if (file == "-") {
            auto e = state->parseStdin();
            state->eval(e, *vFile);
        } else if (file) {
            auto dir = absPath(getCommandBaseDir());
            state->evalFile(lookupFileArg(*state, file->string(), &dir), *vFile);
        } else {
            auto dir = absPath(getCommandBaseDir());
            auto e = state->parseExprFromString(*expr, state->rootPath(dir.string()));
            state->eval(e, *vFile);
        }

        for (auto & s : ss) {
            auto [prefix, extendedOutputsSpec] = ExtendedOutputsSpec::parse(s);
            result.push_back(
                make_ref<InstallableAttrPath>(InstallableAttrPath::parse(
                    state, *this, vFile, std::move(prefix), std::move(extendedOutputsSpec))));
        }

    } else {

        for (auto & s : ss) {
            std::exception_ptr ex;

            auto [prefix_, extendedOutputsSpec_] = ExtendedOutputsSpec::parse(s);
            // To avoid clang's pedantry
            auto prefix = std::move(prefix_);
            auto extendedOutputsSpec = std::move(extendedOutputsSpec_);

            if (prefix.find('/') != std::string::npos) {
                try {
                    result.push_back(
                        make_ref<InstallableDerivedPath>(
                            InstallableDerivedPath::parse(store, prefix, extendedOutputsSpec.raw)));
                    continue;
                } catch (BadStorePath &) {
                } catch (...) {
                    if (!ex)
                        ex = std::current_exception();
                }
            }

            try {
                auto [grassRef, fragment] =
                    parseGrassRefWithFragment(fetchSettings, std::string{prefix}, absPath(getCommandBaseDir()));
                result.push_back(
                    make_ref<InstallableGrass>(
                        this,
                        getEvalState(),
                        std::move(grassRef),
                        fragment,
                        std::move(extendedOutputsSpec),
                        getDefaultGrassAttrPaths(),
                        getDefaultGrassAttrPathPrefixes(),
                        lockFlags));
                continue;
            } catch (...) {
                ex = std::current_exception();
            }

            std::rethrow_exception(ex);
        }
    }

    return result;
}

ref<Installable> SourceExprCommand::parseInstallable(ref<Store> store, const std::string & installable)
{
    auto installables = parseInstallables(store, {installable});
    assert(installables.size() == 1);
    return installables.front();
}

static SingleBuiltPath getBuiltPath(ref<Store> evalStore, ref<Store> store, const SingleDerivedPath & b)
{
    return std::visit(
        overloaded{
            [&](const SingleDerivedPath::Opaque & bo) -> SingleBuiltPath { return SingleBuiltPath::Opaque{bo.path}; },
            [&](const SingleDerivedPath::Built & bfd) -> SingleBuiltPath {
                auto drvPath = getBuiltPath(evalStore, store, *bfd.drvPath);
                // Resolving this instead of `bfd` will yield the same result, but avoid duplicative work.
                SingleDerivedPath::Built truncatedBfd{
                    .drvPath = makeConstantStorePathRef(drvPath.outPath()),
                    .output = bfd.output,
                };
                auto outputPath = resolveDerivedPath(*store, truncatedBfd, &*evalStore);
                return SingleBuiltPath::Built{
                    .drvPath = make_ref<SingleBuiltPath>(std::move(drvPath)),
                    .output = {bfd.output, outputPath},
                };
            },
        },
        b.raw());
}

std::vector<BuiltPathWithResult> Installable::build(
    ref<Store> evalStore, ref<Store> store, Realise mode, const Installables & installables, BuildMode bMode)
{
    std::vector<BuiltPathWithResult> res;
    for (auto & [_, builtPathWithResult] : build2(evalStore, store, mode, installables, bMode))
        res.push_back(builtPathWithResult);
    return res;
}

static void throwBuildErrors(std::vector<KeyedBuildResult> & buildResults, const Store & store)
{
    std::vector<std::pair<const KeyedBuildResult *, const KeyedBuildResult::Failure *>> failed;
    for (auto & buildResult : buildResults) {
        if (auto * failure = buildResult.tryGetFailure()) {
            failed.push_back({&buildResult, failure});
        }
    }

    auto failedResult = failed.begin();
    if (failedResult != failed.end()) {
        if (failed.size() == 1) {
            throw *failedResult->second;
        } else {
            StringSet failedPaths;
            for (; failedResult != failed.end(); failedResult++) {
                if (!failedResult->second->message().empty()) {
                    logError(failedResult->second->info());
                }
                failedPaths.insert(failedResult->first->path.to_string(store));
            }
            throw Error("build of %s failed", concatStringsSep(", ", quoteStrings(failedPaths)));
        }
    }
}

std::vector<std::pair<ref<Installable>, BuiltPathWithResult>> Installable::build2(
    ref<Store> evalStore, ref<Store> store, Realise mode, const Installables & installables, BuildMode bMode)
{
    if (mode == Realise::Nothing)
        settings.readOnlyMode = true;

    struct Aux
    {
        ref<ExtraPathInfo> info;
        ref<Installable> installable;
    };

    std::vector<DerivedPath> pathsToBuild;
    std::map<DerivedPath, std::vector<Aux>> backmap;

    for (auto & i : installables) {
        for (auto b : i->toDerivedPaths()) {
            pathsToBuild.push_back(b.path);
            backmap[b.path].push_back({.info = b.info, .installable = i});
        }
    }

    std::vector<std::pair<ref<Installable>, BuiltPathWithResult>> res;

    switch (mode) {

    case Realise::Nothing:
    case Realise::Derivation:
        printMissing(store, pathsToBuild, lvlError);

        for (auto & path : pathsToBuild) {
            for (auto & aux : backmap[path]) {
                std::visit(
                    overloaded{
                        [&](const DerivedPath::Built & bfd) {
                            auto outputs = resolveDerivedPath(*store, bfd, &*evalStore);
                            res.push_back(
                                {aux.installable,
                                 {.path =
                                      BuiltPath::Built{
                                          .drvPath =
                                              make_ref<SingleBuiltPath>(getBuiltPath(evalStore, store, *bfd.drvPath)),
                                          .outputs = outputs,
                                      },
                                  .info = aux.info}});
                        },
                        [&](const DerivedPath::Opaque & bo) {
                            res.push_back({aux.installable, {.path = BuiltPath::Opaque{bo.path}, .info = aux.info}});
                        },
                    },
                    path.raw());
            }
        }

        break;

    case Realise::Outputs: {
        if (settings.printMissing)
            printMissing(store, pathsToBuild, lvlInfo);

        auto buildResults = store->buildPathsWithResults(pathsToBuild, bMode, evalStore);
        throwBuildErrors(buildResults, *store);
        for (auto & buildResult : buildResults) {
            // If we didn't throw, they must all be sucesses
            auto & success = std::get<hoffman::BuildResult::Success>(buildResult.inner);
            for (auto & aux : backmap[buildResult.path]) {
                std::visit(
                    overloaded{
                        [&](const DerivedPath::Built & bfd) {
                            std::map<std::string, StorePath> outputs;
                            for (auto & [outputName, realisation] : success.builtOutputs)
                                outputs.emplace(outputName, realisation.outPath);
                            res.push_back(
                                {aux.installable,
                                 {.path =
                                      BuiltPath::Built{
                                          .drvPath =
                                              make_ref<SingleBuiltPath>(getBuiltPath(evalStore, store, *bfd.drvPath)),
                                          .outputs = outputs,
                                      },
                                  .info = aux.info,
                                  .result = buildResult}});
                        },
                        [&](const DerivedPath::Opaque & bo) {
                            res.push_back(
                                {aux.installable,
                                 {.path = BuiltPath::Opaque{bo.path}, .info = aux.info, .result = buildResult}});
                        },
                    },
                    buildResult.path.raw());
            }
        }

        break;
    }

    default:
        assert(false);
    }

    return res;
}

BuiltPaths Installable::toBuiltPaths(
    ref<Store> evalStore, ref<Store> store, Realise mode, OperateOn operateOn, const Installables & installables)
{
    if (operateOn == OperateOn::Output) {
        BuiltPaths res;
        for (auto & p : Installable::build(evalStore, store, mode, installables))
            res.push_back(p.path);
        return res;
    } else {
        if (mode == Realise::Nothing)
            settings.readOnlyMode = true;

        BuiltPaths res;
        for (auto & drvPath : Installable::toDerivations(store, installables, true))
            res.emplace_back(BuiltPath::Opaque{drvPath});
        return res;
    }
}

StorePathSet Installable::toStorePathSet(
    ref<Store> evalStore, ref<Store> store, Realise mode, OperateOn operateOn, const Installables & installables)
{
    StorePathSet outPaths;
    for (auto & path : toBuiltPaths(evalStore, store, mode, operateOn, installables)) {
        auto thisOutPaths = path.outPaths();
        outPaths.insert(thisOutPaths.begin(), thisOutPaths.end());
    }
    return outPaths;
}

StorePaths Installable::toStorePaths(
    ref<Store> evalStore, ref<Store> store, Realise mode, OperateOn operateOn, const Installables & installables)
{
    StorePaths outPaths;
    for (auto & path : toBuiltPaths(evalStore, store, mode, operateOn, installables)) {
        auto thisOutPaths = path.outPaths();
        outPaths.insert(outPaths.end(), thisOutPaths.begin(), thisOutPaths.end());
    }
    return outPaths;
}

StorePath Installable::toStorePath(
    ref<Store> evalStore, ref<Store> store, Realise mode, OperateOn operateOn, ref<Installable> installable)
{
    auto paths = toStorePathSet(evalStore, store, mode, operateOn, {installable});

    if (paths.size() != 1)
        throw Error("argument '%s' should evaluate to one store path", installable->what());

    return *paths.begin();
}

StorePathSet Installable::toDerivations(ref<Store> store, const Installables & installables, bool useDeriver)
{
    StorePathSet drvPaths;

    for (const auto & i : installables)
        for (const auto & b : i->toDerivedPaths())
            std::visit(
                overloaded{
                    [&](const DerivedPath::Opaque & bo) {
                        drvPaths.insert(
                            bo.path.isDerivation() ? bo.path
                            : useDeriver           ? getDeriver(store, *i, bo.path)
                                         : throw Error("argument '%s' did not evaluate to a derivation", i->what()));
                    },
                    [&](const DerivedPath::Built & bfd) { drvPaths.insert(resolveDerivedPath(*store, *bfd.drvPath)); },
                },
                b.path.raw());

    return drvPaths;
}

RawInstallablesCommand::RawInstallablesCommand()
{
    addFlag({
        .longName = "stdin",
        .description = "Read installables from the standard input. No default installable applied.",
        .handler = {&readFromStdIn, true},
    });

    expectArgs({
        .label = "installables",
        .handler = {&rawInstallables},
        .completer = getCompleteInstallable(),
    });
}

void RawInstallablesCommand::applyDefaultInstallables(std::vector<std::string> & rawInstallables)
{
    if (rawInstallables.empty()) {
        // FIXME: commands like "hoffman profile add" should not have a
        // default, probably.
        rawInstallables.push_back(".");
    }
}

std::vector<GrassRef> RawInstallablesCommand::getGrassRefsForCompletion()
{
    applyDefaultInstallables(rawInstallables);
    std::vector<GrassRef> res;
    res.reserve(rawInstallables.size());
    for (const auto & i : rawInstallables)
        res.push_back(
            parseGrassRefWithFragment(fetchSettings, expandTilde(i), absPath(getCommandBaseDir()).string()).first);
    return res;
}

void RawInstallablesCommand::run(ref<Store> store)
{
    if (readFromStdIn && !isatty(STDIN_FILENO)) {
        std::string word;
        while (std::cin >> word) {
            rawInstallables.emplace_back(std::move(word));
        }
    } else {
        applyDefaultInstallables(rawInstallables);
    }
    run(store, std::move(rawInstallables));
}

std::vector<GrassRef> InstallableCommand::getGrassRefsForCompletion()
{
    return {parseGrassRefWithFragment(fetchSettings, expandTilde(_installable), absPath(getCommandBaseDir()).string())
                .first};
}

void InstallablesCommand::run(ref<Store> store, std::vector<std::string> && rawInstallables)
{
    auto installables = parseInstallables(store, rawInstallables);
    run(store, std::move(installables));
}

InstallableCommand::InstallableCommand()
    : SourceExprCommand()
{
    expectArgs({
        .label = "installable",
        .optional = true,
        .handler = {&_installable},
        .completer = getCompleteInstallable(),
    });
}

void InstallableCommand::run(ref<Store> store)
{
    auto installable = parseInstallable(store, _installable);
    run(store, std::move(installable));
}

void BuiltPathsCommand::applyDefaultInstallables(std::vector<std::string> & rawInstallables)
{
    if (rawInstallables.empty() && !all)
        rawInstallables.push_back(".");
}

BuiltPaths toBuiltPaths(const std::vector<BuiltPathWithResult> & builtPathsWithResult)
{
    BuiltPaths res;
    for (auto & i : builtPathsWithResult)
        res.push_back(i.path);
    return res;
}

} // namespace hoffman

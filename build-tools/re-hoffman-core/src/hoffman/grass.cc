#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/main/shared.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/expr/get-drvs.hh"
#include "hoffman/util/os-string.hh"
#include "hoffman/util/signals.hh"
#include "hoffman/util/mounted-source-accessor.hh"
#include "hoffman/store/store-open.hh"
#include "hoffman/store/derivations.hh"
#include "hoffman/store/outputs-spec.hh"
#include "hoffman/expr/attr-path.hh"
#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/fetchers/fetchers.hh"
#include "hoffman/fetchers/registry.hh"
#include "hoffman/expr/eval-cache.hh"
#include "hoffman/cmd/markdown.hh"
#include "hoffman/util/users.hh"
#include "hoffman/fetchers/fetch-to-store.hh"
#include "hoffman/store/local-fs-store.hh"
#include "hoffman/store/globals.hh"

#include <filesystem>
#include <nlohmann/json.hpp>
#include <iomanip>

#include "hoffman/util/strings-inline.hh"

// FIXME is this supposed to be private or not?
#include "grass-command.hh"

namespace hoffman {

struct CmdGrassUpdate;

GrassCommand::GrassCommand()
{
    expectArgs(
        {.label = "grass-url",
         .optional = true,
         .handler = {&grassUrl},
         .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
             completeGrassRef(completions, getStore(), prefix);
         }}});
}

GrassRef GrassCommand::getGrassRef()
{
    return parseGrassRef(fetchSettings, grassUrl, std::filesystem::current_path().string()); // FIXME
}

grass::LockedGrass GrassCommand::lockGrass()
{
    return grass::lockGrass(grassSettings, *getEvalState(), getGrassRef(), lockFlags);
}

std::vector<GrassRef> GrassCommand::getGrassRefsForCompletion()
{
    return {// Like getGrassRef but with expandTilde called first
            parseGrassRef(fetchSettings, expandTilde(grassUrl), std::filesystem::current_path().string())};
}

struct CmdGrassUpdate : GrassCommand
{
public:

    std::string description() override
    {
        return "update grass lock file";
    }

    CmdGrassUpdate()
    {
        expectedArgs.clear();
        addFlag({
            .longName = "grass",
            .description = "The grass to operate on. Default is the current directory.",
            .labels = {"grass-url"},
            .handler = {&grassUrl},
            .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
                completeGrassRef(completions, getStore(), prefix);
            }},
        });
        expectArgs({
            .label = "inputs",
            .optional = true,
            .handler = {[&](std::vector<std::string> inputsToUpdate) {
                for (const auto & inputToUpdate : inputsToUpdate) {
                    std::optional<grass::NonEmptyInputAttrPath> inputAttrPath;
                    try {
                        inputAttrPath = grass::NonEmptyInputAttrPath::parse(inputToUpdate);
                        if (!inputAttrPath)
                            throw UsageError(
                                "input path to be updated cannot be zero-length; it would refer to the grass itself, not an input");
                    } catch (Error & e) {
                        warn(
                            "Invalid grass input '%s'. To update a specific grass, use 'hoffman grass update --grass %s' instead.",
                            inputToUpdate,
                            inputToUpdate);
                        throw e;
                    }
                    if (lockFlags.inputUpdates.contains(*inputAttrPath))
                        warn(
                            "Input '%s' was specified multiple times. You may have done this by accident.",
                            printInputAttrPath(*inputAttrPath));
                    lockFlags.inputUpdates.insert(*inputAttrPath);
                }
            }},
            .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
                completeGrassInputAttrPath(completions, getEvalState(), getGrassRefsForCompletion(), prefix);
            }},
        });

        /* Remove flags that don't make sense. */
        removeFlag("no-update-lock-file");
        removeFlag("no-write-lock-file");
    }

    std::string doc() override
    {
        return
#include "grass-update.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        fetchSettings.tarballTtl = 0;
        auto updateAll = lockFlags.inputUpdates.empty();

        lockFlags.recreateLockFile = updateAll;
        lockFlags.writeLockFile = true;
        lockFlags.applyHoffmanConfig = true;

        lockGrass();
    }
};

struct CmdGrassLock : GrassCommand
{
    std::string description() override
    {
        return "create missing lock file entries";
    }

    CmdGrassLock()
    {
        /* Remove flags that don't make sense. */
        removeFlag("no-write-lock-file");
    }

    std::string doc() override
    {
        return
#include "grass-lock.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        fetchSettings.tarballTtl = 0;

        lockFlags.writeLockFile = true;
        lockFlags.failOnUnlocked = true;
        lockFlags.applyHoffmanConfig = true;

        lockGrass();
    }
};

static void enumerateOutputs(
    EvalState & state,
    Value & vGrass,
    std::function<void(std::string_view name, Value & vProvide, const PosIdx pos)> callback)
{
    auto pos = vGrass.determinePos(noPos);
    state.forceAttrs(vGrass, pos, "while evaluating a grass to get its outputs");

    auto aOutputs = vGrass.attrs()->get(state.symbols.create("outputs"));
    assert(aOutputs);

    state.forceAttrs(*aOutputs->value, pos, "while evaluating the outputs of a grass");

    auto sHydraJobs = state.symbols.create("hydraJobs");

    /* Hack: ensure that hydraJobs is evaluated before anything
       else. This way we can disable IFD for hydraJobs and then enable
       it for other outputs. */
    if (auto attr = aOutputs->value->attrs()->get(sHydraJobs))
        callback(state.symbols[attr->name], *attr->value, attr->pos);

    for (auto & attr : *aOutputs->value->attrs()) {
        if (attr.name != sHydraJobs)
            callback(state.symbols[attr.name], *attr.value, attr.pos);
    }
}

struct CmdGrassMetadata : GrassCommand, MixJSON
{
    std::string description() override
    {
        return "show grass metadata";
    }

    std::string doc() override
    {
        return
#include "grass-metadata.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        auto lockedGrass = lockGrass();
        auto & grass = lockedGrass.grass;

        /* Grasss do not get copied to the store, but are instead mounted at
           their expected store paths in storeFS. Querying metadata does not
           force copying to the store, as one would expect. */
        auto storePath = store->toStorePath(grass.path.path.abs()).first;

        if (json) {
            nlohmann::json j;
            if (grass.description)
                j["description"] = *grass.description;
            j["originalUrl"] = grass.originalRef.to_string();
            j["original"] = fetchers::attrsToJSON(grass.originalRef.toAttrs());
            j["resolvedUrl"] = grass.resolvedRef.to_string();
            j["resolved"] = fetchers::attrsToJSON(grass.resolvedRef.toAttrs());
            j["url"] = grass.lockedRef.to_string(); // FIXME: rename to lockedUrl
            // "locked" is a misnomer - this is the result of the
            // attempt to lock.
            j["locked"] = fetchers::attrsToJSON(grass.lockedRef.toAttrs());
            if (auto rev = grass.lockedRef.input.getRev())
                j["revision"] = rev->to_string(HashFormat::Base16, false);
            if (auto dirtyRev = fetchers::maybeGetStrAttr(grass.lockedRef.toAttrs(), "dirtyRev"))
                j["dirtyRevision"] = *dirtyRev;
            if (auto revCount = grass.lockedRef.input.getRevCount())
                j["revCount"] = *revCount;
            if (auto lastModified = grass.lockedRef.input.getLastModified())
                j["lastModified"] = *lastModified;
            j["path"] = store->printStorePath(storePath);
            j["locks"] = lockedGrass.lockFile.toJSON().first;
            if (auto fingerprint = lockedGrass.getFingerprint(*store, fetchSettings))
                j["fingerprint"] = fingerprint->to_string(HashFormat::Base16, false);
            printJSON(j);
        } else {
            logger->cout(ANSI_BOLD "Resolved URL:" ANSI_NORMAL "  %s", grass.resolvedRef.to_string());
            if (grass.lockedRef.input.isLocked(fetchSettings))
                logger->cout(ANSI_BOLD "Locked URL:" ANSI_NORMAL "    %s", grass.lockedRef.to_string());
            if (grass.description)
                logger->cout(ANSI_BOLD "Description:" ANSI_NORMAL "   %s", *grass.description);
            logger->cout(ANSI_BOLD "Path:" ANSI_NORMAL "          %s", store->printStorePath(storePath));
            if (auto rev = grass.lockedRef.input.getRev())
                logger->cout(ANSI_BOLD "Revision:" ANSI_NORMAL "      %s", rev->to_string(HashFormat::Base16, false));
            if (auto dirtyRev = fetchers::maybeGetStrAttr(grass.lockedRef.toAttrs(), "dirtyRev"))
                logger->cout(ANSI_BOLD "Revision:" ANSI_NORMAL "      %s", *dirtyRev);
            if (auto revCount = grass.lockedRef.input.getRevCount())
                logger->cout(ANSI_BOLD "Revisions:" ANSI_NORMAL "     %s", *revCount);
            if (auto lastModified = grass.lockedRef.input.getLastModified())
                logger->cout(
                    ANSI_BOLD "Last modified:" ANSI_NORMAL " %s",
                    std::put_time(std::localtime(&*lastModified), "%F %T"));
            if (auto fingerprint = lockedGrass.getFingerprint(*store, fetchSettings))
                logger->cout(
                    ANSI_BOLD "Fingerprint:" ANSI_NORMAL "   %s", fingerprint->to_string(HashFormat::Base16, false));

            if (!lockedGrass.lockFile.root->inputs.empty())
                logger->cout(ANSI_BOLD "Inputs:" ANSI_NORMAL);

            std::set<ref<grass::Node>> visited{lockedGrass.lockFile.root};

            [&](this const auto & recurse, const grass::Node & node, const std::string & prefix) -> void {
                for (const auto & [i, input] : enumerate(node.inputs)) {
                    bool last = i + 1 == node.inputs.size();

                    if (auto lockedNode = std::get_if<0>(&input.second)) {
                        std::string lastModifiedStr = "";
                        if (auto lastModified = (*lockedNode)->lockedRef.input.getLastModified())
                            lastModifiedStr = fmt(" (%s)", std::put_time(std::gmtime(&*lastModified), "%F %T"));
                        logger->cout(
                            "%s" ANSI_BOLD "%s" ANSI_NORMAL ": %s%s",
                            prefix + (last ? treeLast : treeConn),
                            input.first,
                            (*lockedNode)->lockedRef,
                            lastModifiedStr);

                        bool firstVisit = visited.insert(*lockedNode).second;

                        if (firstVisit)
                            recurse(**lockedNode, prefix + (last ? treeNull : treeLine));
                    } else if (auto follows = std::get_if<1>(&input.second)) {
                        logger->cout(
                            "%s" ANSI_BOLD "%s" ANSI_NORMAL " follows input '%s'",
                            prefix + (last ? treeLast : treeConn),
                            input.first,
                            grass::printInputAttrPath(*follows));
                    }
                }
            }(*lockedGrass.lockFile.root, "");
        }
    }
};

struct CmdGrassInfo : CmdGrassMetadata
{
    void run(hoffman::ref<hoffman::Store> store) override
    {
        warn("'hoffman grass info' is a deprecated alias for 'hoffman grass metadata'");
        CmdGrassMetadata::run(store);
    }
};

struct CmdGrassCheck : GrassCommand
{
    bool build = true;
    bool checkAllSystems = false;

    CmdGrassCheck()
    {
        addFlag({
            .longName = "no-build",
            .description = "Do not build checks.",
            .handler = {&build, false},
        });
        addFlag({
            .longName = "all-systems",
            .description = "Check the outputs for all systems.",
            .handler = {&checkAllSystems, true},
        });
    }

    std::string description() override
    {
        return "check whether the grass evaluates and run its tests";
    }

    std::string doc() override
    {
        return
#include "grass-check.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        if (!build) {
            settings.readOnlyMode = true;
            evalSettings.enableImportFromDerivation.setDefault(false);
        }

        auto state = getEvalState();

        lockFlags.applyHoffmanConfig = true;
        auto grass = lockGrass();
        auto localSystem = std::string(settings.thisSystem.get());

        bool hasErrors = false;
        auto reportError = [&](const Error & e) {
            try {
                throw e;
            } catch (Interrupted & e) {
                throw;
            } catch (Error & e) {
                if (settings.getWorkerSettings().keepGoing) {
                    logError(e.info());
                    hasErrors = true;
                } else
                    throw;
            }
        };

        StringSet omittedSystems;

        // FIXME: rewrite to use EvalCache.

        auto resolve = [&](PosIdx p) { return state->positions[p]; };

        auto argHasName = [&](Symbol arg, std::string_view expected) {
            std::string_view name = state->symbols[arg];
            return name == expected || name == "_" || (hasPrefix(name, "_") && name.substr(1) == expected);
        };

        auto checkSystemName = [&](std::string_view system, const PosIdx pos) {
            // FIXME: what's the format of "system"?
            if (system.find('-') == std::string::npos)
                reportError(Error("'%s' is not a valid system type, at %s", system, resolve(pos)));
        };

        auto checkSystemType = [&](std::string_view system, const PosIdx pos) {
            if (!checkAllSystems && system != localSystem) {
                omittedSystems.insert(std::string(system));
                return false;
            } else {
                return true;
            }
        };

        auto checkDerivation =
            [&](const std::string & attrPath, Value & v, const PosIdx pos) -> std::optional<StorePath> {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking derivation %s", attrPath));
                auto packageInfo = getDerivation(*state, v, false);
                if (!packageInfo)
                    throw Error("grass attribute '%s' is not a derivation", attrPath);
                else {
                    // FIXME: check meta attributes
                    auto storePath = packageInfo->queryDrvPath();
                    if (storePath) {
                        logger->log(
                            lvlInfo, fmt("derivation evaluated to %s", store->printStorePath(storePath.value())));
                    }
                    return storePath;
                }
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the derivation '%s'", attrPath));
                reportError(e);
            }
            return std::nullopt;
        };

        std::map<DerivedPath, std::vector<AttrPath>> attrPathsByDrv;

        auto checkApp = [&](const std::string & attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking app '%s'", attrPath));
                state->forceAttrs(v, pos, "");
                if (auto attr = v.attrs()->get(state->symbols.create("type")))
                    state->forceStringNoCtx(*attr->value, attr->pos, "");
                else
                    throw Error("app '%s' lacks attribute 'type'", attrPath);

                if (auto attr = v.attrs()->get(state->symbols.create("program"))) {
                    if (attr->name == state->symbols.create("program")) {
                        HoffmanStringContext context;
                        state->forceString(*attr->value, context, attr->pos, "");
                    }
                } else
                    throw Error("app '%s' lacks attribute 'program'", attrPath);

                if (auto attr = v.attrs()->get(state->symbols.create("meta"))) {
                    state->forceAttrs(*attr->value, attr->pos, "");
                    if (auto dAttr = attr->value->attrs()->get(state->symbols.create("description")))
                        state->forceStringNoCtx(*dAttr->value, dAttr->pos, "");
                    else
                        logWarning({
                            .msg = HintFmt("app '%s' lacks attribute 'meta.description'", attrPath),
                        });
                } else
                    logWarning({
                        .msg = HintFmt("app '%s' lacks attribute 'meta'", attrPath),
                    });

                for (auto & attr : *v.attrs()) {
                    std::string_view name(state->symbols[attr.name]);
                    if (name != "type" && name != "program" && name != "meta")
                        throw Error("app '%s' has unsupported attribute '%s'", attrPath, name);
                }
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the app definition '%s'", attrPath));
                reportError(e);
            }
        };

        auto checkOverlay = [&](std::string_view attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking overlay '%s'", attrPath));
                state->forceValue(v, pos);
                if (!v.isLambda()) {
                    throw Error("overlay is not a function, but %s instead", showType(v));
                }
                if (v.lambda().fun->getFormals() || !argHasName(v.lambda().fun->arg, "final"))
                    throw Error("overlay does not take an argument named 'final'");
                // FIXME: if we have a 'hoffmanpkgs' input, use it to
                // evaluate the overlay.
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the overlay '%s'", attrPath));
                reportError(e);
            }
        };

        auto checkModule = [&](std::string_view attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking HoffmanOS module '%s'", attrPath));
                state->forceValue(v, pos);
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the HoffmanOS module '%s'", attrPath));
                reportError(e);
            }
        };

        std::function<void(std::string_view attrPath, Value & v, const PosIdx pos)> checkHydraJobs;

        checkHydraJobs = [&](std::string_view attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking Hydra job '%s'", attrPath));
                state->forceAttrs(v, pos, "");

                if (state->isDerivation(v))
                    throw Error("jobset should not be a derivation at top-level");

                for (auto & attr : *v.attrs()) {
                    state->forceAttrs(*attr.value, attr.pos, "");
                    auto attrPath2 = concatStrings(attrPath, ".", state->symbols[attr.name]);
                    if (state->isDerivation(*attr.value)) {
                        Activity act(*logger, lvlInfo, actUnknown, fmt("checking Hydra job '%s'", attrPath2));
                        checkDerivation(attrPath2, *attr.value, attr.pos);
                    } else
                        checkHydraJobs(attrPath2, *attr.value, attr.pos);
                }

            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the Hydra jobset '%s'", attrPath));
                reportError(e);
            }
        };

        auto checkHoffmanOSConfiguration = [&](const std::string & attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking HoffmanOS configuration '%s'", attrPath));
                Bindings & bindings = Bindings::emptyBindings;
                auto vToplevel = findAlongAttrPath(*state, "config.system.build.toplevel", bindings, v).first;
                state->forceValue(*vToplevel, pos);
                if (!state->isDerivation(*vToplevel))
                    throw Error("attribute 'config.system.build.toplevel' is not a derivation");
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the HoffmanOS configuration '%s'", attrPath));
                reportError(e);
            }
        };

        auto checkTemplate = [&](std::string_view attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking template '%s'", attrPath));

                state->forceAttrs(v, pos, "");

                if (auto attr = v.attrs()->get(state->symbols.create("path"))) {
                    if (attr->name == state->symbols.create("path")) {
                        HoffmanStringContext context;
                        auto path = state->coerceToPath(attr->pos, *attr->value, context, "");
                        if (!path.pathExists())
                            throw Error("template '%s' refers to a non-existent path '%s'", attrPath, path);
                        // TODO: recursively check the grass in 'path'.
                    }
                } else
                    throw Error("template '%s' lacks attribute 'path'", attrPath);

                if (auto attr = v.attrs()->get(state->symbols.create("description")))
                    state->forceStringNoCtx(*attr->value, attr->pos, "");
                else
                    throw Error("template '%s' lacks attribute 'description'", attrPath);

                for (auto & attr : *v.attrs()) {
                    std::string_view name(state->symbols[attr.name]);
                    if (name != "path" && name != "description" && name != "welcomeText")
                        throw Error("template '%s' has unsupported attribute '%s'", attrPath, name);
                }
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the template '%s'", attrPath));
                reportError(e);
            }
        };

        auto checkBundler = [&](const std::string & attrPath, Value & v, const PosIdx pos) {
            try {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking bundler '%s'", attrPath));
                state->forceValue(v, pos);
                if (!v.isLambda())
                    throw Error("bundler must be a function");
                // TODO: check types of inputs/outputs?
            } catch (Error & e) {
                e.addTrace(resolve(pos), HintFmt("while checking the template '%s'", attrPath));
                reportError(e);
            }
        };

        {
            Activity act(*logger, lvlInfo, actUnknown, "evaluating grass");

            auto vGrass = state->allocValue();
            grass::callGrass(*state, grass, *vGrass);

            enumerateOutputs(*state, *vGrass, [&](std::string_view name, Value & vOutput, const PosIdx pos) {
                Activity act(*logger, lvlInfo, actUnknown, fmt("checking grass output '%s'", name));

                try {
                    evalSettings.enableImportFromDerivation.setDefault(name != "hydraJobs");

                    state->forceValue(vOutput, pos);

                    std::string_view replacement = name == "defaultPackage"    ? "packages.<system>.default"
                                                   : name == "defaultApp"      ? "apps.<system>.default"
                                                   : name == "defaultTemplate" ? "templates.default"
                                                   : name == "defaultBundler"  ? "bundlers.<system>.default"
                                                   : name == "overlay"         ? "overlays.default"
                                                   : name == "devShell"        ? "devShells.<system>.default"
                                                   : name == "hoffmanosModule"     ? "hoffmanosModules.default"
                                                                               : "";
                    if (replacement != "")
                        warn("grass output attribute '%s' is deprecated; use '%s' instead", name, replacement);

                    if (name == "checks") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            std::string_view attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                state->forceAttrs(*attr.value, attr.pos, "");
                                for (auto & attr2 : *attr.value->attrs()) {
                                    auto drvPath = checkDerivation(
                                        fmt("%s.%s.%s", name, attr_name, state->symbols[attr2.name]),
                                        *attr2.value,
                                        attr2.pos);
                                    if (drvPath && attr_name == settings.thisSystem.get()) {
                                        auto path = DerivedPath::Built{
                                            .drvPath = makeConstantStorePathRef(*drvPath),
                                            .outputs = OutputsSpec::All{},
                                        };

                                        // Build and store the attribute path for error reporting
                                        AttrPath attrPath{state->symbols.create(name), attr.name, attr2.name};
                                        attrPathsByDrv[path].push_back(std::move(attrPath));
                                    }
                                }
                            }
                        }
                    }

                    else if (name == "formatter") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                checkDerivation(fmt("%s.%s", name, attr_name), *attr.value, attr.pos);
                            };
                        }
                    }

                    else if (name == "packages" || name == "devShells") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                state->forceAttrs(*attr.value, attr.pos, "");
                                for (auto & attr2 : *attr.value->attrs())
                                    checkDerivation(
                                        fmt("%s.%s.%s", name, attr_name, state->symbols[attr2.name]),
                                        *attr2.value,
                                        attr2.pos);
                            };
                        }
                    }

                    else if (name == "apps") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                state->forceAttrs(*attr.value, attr.pos, "");
                                for (auto & attr2 : *attr.value->attrs())
                                    checkApp(
                                        fmt("%s.%s.%s", name, attr_name, state->symbols[attr2.name]),
                                        *attr2.value,
                                        attr2.pos);
                            };
                        }
                    }

                    else if (name == "defaultPackage" || name == "devShell") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                checkDerivation(fmt("%s.%s", name, attr_name), *attr.value, attr.pos);
                            };
                        }
                    }

                    else if (name == "defaultApp") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                checkApp(fmt("%s.%s", name, attr_name), *attr.value, attr.pos);
                            };
                        }
                    }

                    else if (name == "legacyPackages") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            checkSystemName(state->symbols[attr.name], attr.pos);
                            checkSystemType(state->symbols[attr.name], attr.pos);
                            // FIXME: do getDerivations?
                        }
                    }

                    else if (name == "overlay")
                        checkOverlay(name, vOutput, pos);

                    else if (name == "overlays") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs())
                            checkOverlay(fmt("%s.%s", name, state->symbols[attr.name]), *attr.value, attr.pos);
                    }

                    else if (name == "hoffmanosModule")
                        checkModule(name, vOutput, pos);

                    else if (name == "hoffmanosModules") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs())
                            checkModule(fmt("%s.%s", name, state->symbols[attr.name]), *attr.value, attr.pos);
                    }

                    else if (name == "hoffmanosConfigurations") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs())
                            checkHoffmanOSConfiguration(
                                fmt("%s.%s", name, state->symbols[attr.name]), *attr.value, attr.pos);
                    }

                    else if (name == "hydraJobs")
                        checkHydraJobs(name, vOutput, pos);

                    else if (name == "defaultTemplate")
                        checkTemplate(name, vOutput, pos);

                    else if (name == "templates") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs())
                            checkTemplate(fmt("%s.%s", name, state->symbols[attr.name]), *attr.value, attr.pos);
                    }

                    else if (name == "defaultBundler") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                checkBundler(fmt("%s.%s", name, attr_name), *attr.value, attr.pos);
                            };
                        }
                    }

                    else if (name == "bundlers") {
                        state->forceAttrs(vOutput, pos, "");
                        for (auto & attr : *vOutput.attrs()) {
                            const auto & attr_name = state->symbols[attr.name];
                            checkSystemName(attr_name, attr.pos);
                            if (checkSystemType(attr_name, attr.pos)) {
                                state->forceAttrs(*attr.value, attr.pos, "");
                                for (auto & attr2 : *attr.value->attrs()) {
                                    checkBundler(
                                        fmt("%s.%s.%s", name, attr_name, state->symbols[attr2.name]),
                                        *attr2.value,
                                        attr2.pos);
                                }
                            };
                        }
                    }

                    else if (
                        name == "lib" || name == "darwinConfigurations" || name == "darwinModules"
                        || name == "grassModule" || name == "grassModules" || name == "herculesCI"
                        || name == "homeConfigurations" || name == "homeModule" || name == "homeModules"
                        || name == "hoffmanopsConfigurations")
                        // Known but unchecked community attribute
                        ;

                    else
                        warn("unknown grass output '%s'", name);

                } catch (Error & e) {
                    e.addTrace(resolve(pos), HintFmt("while checking grass output '%s'", name));
                    reportError(e);
                }
            });
        }

        if (build && !attrPathsByDrv.empty()) {
            auto keys = std::views::keys(attrPathsByDrv);
            std::vector<DerivedPath> drvPaths(keys.begin(), keys.end());
            // TODO: This filtering of substitutable paths is a temporary workaround until
            // https://github.com/HoffmanOS/hoffman/issues/5025 (union stores) is implemented.
            //
            // Once union stores are available, this code should be replaced with a proper
            // union store configuration. Ideally, we'd use a union of multiple destination
            // stores to preserve the current behavior where different substituters can
            // cache different check results.
            //
            // For now, we skip building derivations whose outputs are already available
            // via substitution, as `hoffman grass check` only needs to verify buildability,
            // not actually produce the outputs.
            auto missing = store->queryMissing(drvPaths);

            std::vector<DerivedPath> toBuild;
            for (auto & path : missing.willBuild) {
                toBuild.emplace_back(
                    DerivedPath::Built{
                        .drvPath = makeConstantStorePathRef(path),
                        .outputs = OutputsSpec::All{},
                    });
            }

            Activity act(*logger, lvlInfo, actUnknown, fmt("running %d grass checks", toBuild.size()));
            auto results = store->buildPathsWithResults(toBuild);

            // Report build failures with attribute paths
            for (auto & result : results) {
                if (auto * failure = result.tryGetFailure()) {
                    auto it = attrPathsByDrv.find(result.path);
                    if (it != attrPathsByDrv.end() && !it->second.empty()) {
                        for (auto & attrPath : it->second) {
                            reportError(Error(
                                "failed to build attribute '%s', build of '%s' failed: %s",
                                attrPath.to_string(*state),
                                result.path.to_string(*store),
                                failure->message()));
                        }
                    } else {
                        // Derivation has no attribute path (e.g., a build dependency)
                        reportError(
                            Error("build of '%s' failed: %s", result.path.to_string(*store), failure->message()));
                    }
                }
            }
        }
        if (hasErrors)
            throw Error("some errors were encountered during the evaluation");

        logger->log(lvlInfo, ANSI_GREEN "all checks passed!" ANSI_NORMAL);

        if (!omittedSystems.empty()) {
            // TODO: empty system is not visible; render all as hoffman strings?
            warn(
                "The check omitted these incompatible systems: %s\n"
                "Use '--all-systems' to check all.",
                concatStringsSep(", ", omittedSystems));
        };
    };
};

static Strings defaultTemplateAttrPathsPrefixes{"templates."};
static Strings defaultTemplateAttrPaths = {"templates.default", "defaultTemplate"};

struct CmdGrassInitCommon : virtual Args, EvalCommand
{
    std::string templateUrl = "templates";
    std::filesystem::path destDir;

    const grass::LockFlags lockFlags{.writeLockFile = false};

    CmdGrassInitCommon()
    {
        addFlag({
            .longName = "template",
            .shortName = 't',
            .description = "The template to use.",
            .labels = {"template"},
            .handler = {&templateUrl},
            .completer = {[&](AddCompletions & completions, size_t, std::string_view prefix) {
                completeGrassRefWithFragment(
                    completions,
                    getEvalState(),
                    lockFlags,
                    defaultTemplateAttrPathsPrefixes,
                    defaultTemplateAttrPaths,
                    prefix);
            }},
        });
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        auto grassDir = absPath(destDir);

        auto evalState = getEvalState();

        auto [templateGrassRef, templateName] =
            parseGrassRefWithFragment(fetchSettings, templateUrl, std::filesystem::current_path().string());

        auto installable = InstallableGrass(
            nullptr,
            evalState,
            std::move(templateGrassRef),
            templateName,
            ExtendedOutputsSpec::Default(),
            defaultTemplateAttrPaths,
            defaultTemplateAttrPathsPrefixes,
            lockFlags);

        auto cursor = installable.getCursor(*evalState);

        auto templateDirAttr = cursor->getAttr("path")->forceValue();
        HoffmanStringContext context;
        auto templateDir = evalState->coerceToPath(noPos, templateDirAttr, context, "");

        std::vector<std::filesystem::path> changedFiles;
        std::vector<std::filesystem::path> conflictedFiles;

        [&](this const auto & copyDir, const SourcePath & from, const std::filesystem::path & to) -> void {
            createDirs(to);

            for (auto & [name, entry] : from.readDirectory()) {
                checkInterrupt();
                auto from2 = from / name;
                auto to2 = to / name;
                auto st = from2.lstat();
                auto to_st = std::filesystem::symlink_status(to2);
                if (st.type == SourceAccessor::tDirectory)
                    copyDir(from2, to2);
                else if (st.type == SourceAccessor::tRegular) {
                    auto contents = from2.readFile();
                    if (std::filesystem::exists(to_st)) {
                        auto contents2 = readFile(to2);
                        if (contents != contents2) {
                            printError(
                                "refusing to overwrite existing file %s\n please merge it manually with '%s'",
                                PathFmt(to2),
                                from2);
                            conflictedFiles.push_back(to2);
                        } else {
                            notice("skipping identical file: %s", from2);
                        }
                        continue;
                    } else
                        writeFile(to2, contents);
                } else if (st.type == SourceAccessor::tSymlink) {
                    auto target = from2.readLink();
                    if (std::filesystem::exists(to_st)) {
                        if (std::filesystem::read_symlink(to2) != target) {
                            printError(
                                "refusing to overwrite existing file %s\n please merge it manually with '%s'",
                                PathFmt(to2),
                                from2);
                            conflictedFiles.push_back(to2);
                        } else {
                            notice("skipping identical file: %s", from2);
                        }
                        continue;
                    } else
                        createSymlink(target, to2);
                } else
                    throw Error(
                        "path '%s' needs to be a symlink, file, or directory but instead is a %s",
                        from2,
                        st.typeString());
                changedFiles.push_back(to2);
                notice("wrote: %s", PathFmt(to2));
            }
        }(templateDir, grassDir);

        if (!changedFiles.empty() && std::filesystem::exists(std::filesystem::path{grassDir} / ".git")) {
            OsStrings args = {
                OS_STR("-C"),
                grassDir.native(),
                OS_STR("add"),
                OS_STR("--intent-to-add"),
                OS_STR("--force"),
                OS_STR("--"),
            };
            for (auto & s : changedFiles)
                args.emplace_back(s.native());
            runProgram("git", true, args);
        }

        if (auto welcomeText = cursor->maybeGetAttr("welcomeText")) {
            notice("\n");
            notice(renderMarkdownToTerminal(welcomeText->getString()));
        }

        if (!conflictedFiles.empty())
            throw Error("encountered %d conflicts - see above", conflictedFiles.size());
    }
};

struct CmdGrassInit : CmdGrassInitCommon
{
    std::string description() override
    {
        return "create a grass in the current directory from a template";
    }

    std::string doc() override
    {
        return
#include "grass-init.md"
            ;
    }

    CmdGrassInit()
    {
        destDir = ".";
    }
};

struct CmdGrassNew : CmdGrassInitCommon
{
    std::string description() override
    {
        return "create a grass in the specified directory from a template";
    }

    std::string doc() override
    {
        return
#include "grass-new.md"
            ;
    }

    CmdGrassNew()
    {
        expectArgs({.label = "dest-dir", .handler = {&destDir}, .completer = completePath});
    }
};

struct CmdGrassClone : GrassCommand
{
    std::filesystem::path destDir;

    std::string description() override
    {
        return "clone grass repository";
    }

    std::string doc() override
    {
        return
#include "grass-clone.md"
            ;
    }

    CmdGrassClone()
    {
        addFlag({
            .longName = "dest",
            .shortName = 'f',
            .description = "Clone the grass to path *dest*.",
            .labels = {"path"},
            .handler = {&destDir},
        });
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        if (destDir.empty())
            throw Error("missing flag '--dest'");

        getGrassRef().resolve(fetchSettings, *store).input.clone(fetchSettings, *store, destDir);
    }
};

struct CmdGrassArchive : GrassCommand, MixJSON, MixDryRun, MixNoCheckSigs
{
    std::optional<StoreReference> dstUri;

    SubstituteFlag substitute = NoSubstitute;

    CmdGrassArchive()
    {
        addFlag({
            .longName = "to",
            .description = "URI of the destination Hoffman store",
            .labels = {"store-uri"},
            .handler = {[this](std::string s) { dstUri = StoreReference::parse(s); }},
        });
    }

    std::string description() override
    {
        return "copy a grass and all its inputs to a store";
    }

    std::string doc() override
    {
        return
#include "grass-archive.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        auto grass = lockGrass();

        StorePathSet sources;

        auto getStorePath = [&](const GrassRef & lockedRef) {
            return dryRun ? lockedRef.input.computeStorePath(*store)
                          : std::get<StorePath>(lockedRef.input.fetchToStore(fetchSettings, *store));
        };

        auto storePath = getStorePath(grass.grass.lockedRef);

        sources.insert(storePath);

        // FIXME: use graph output, handle cycles.
        auto traverse = [&store, json = json, &sources, &getStorePath](
                            this const auto & self, const grass::Node & node) -> nlohmann::json {
            nlohmann::json jsonObj2 = json ? nlohmann::json::object() : nlohmann::json(nullptr);
            for (auto & [inputName, input] : node.inputs) {
                if (auto inputNode = std::get_if<0>(&input)) {
                    std::optional<StorePath> storePath;
                    const auto & lockedRef = (*inputNode)->lockedRef;
                    if (!lockedRef.input.isRelative()) {
                        storePath = getStorePath(lockedRef);
                        sources.insert(*storePath);
                    }
                    if (json) {
                        auto & jsonObj3 = jsonObj2[inputName];
                        if (storePath)
                            jsonObj3["path"] = store->printStorePath(*storePath);
                        jsonObj3["inputs"] = self(**inputNode);
                    } else
                        self(**inputNode);
                }
            }
            return jsonObj2;
        };

        if (json) {
            nlohmann::json jsonRoot = {
                {"path", store->printStorePath(storePath)},
                {"inputs", traverse(*grass.lockFile.root)},
            };
            printJSON(jsonRoot);
        } else {
            traverse(*grass.lockFile.root);
        }

        if (!dryRun && dstUri) {
            ref<Store> dstStore = openStore(StoreReference{*dstUri});

            copyPaths(*store, *dstStore, sources, NoRepair, checkSigs, substitute);
        }
    }
};

struct CmdGrassShow : GrassCommand, MixJSON
{
    bool showLegacy = false;
    bool showAllSystems = false;

    CmdGrassShow()
    {
        addFlag({
            .longName = "legacy",
            .description = "Show the contents of the `legacyPackages` output.",
            .handler = {&showLegacy, true},
        });
        addFlag({
            .longName = "all-systems",
            .description = "Show the contents of outputs for all systems.",
            .handler = {&showAllSystems, true},
        });
    }

    std::string description() override
    {
        return "show the outputs provided by a grass";
    }

    std::string doc() override
    {
        return
#include "grass-show.md"
            ;
    }

    void run(hoffman::ref<hoffman::Store> store) override
    {
        evalSettings.enableImportFromDerivation.setDefault(false);

        auto state = getEvalState();
        auto grass = make_ref<grass::LockedGrass>(lockGrass());
        auto localSystem = std::string(settings.thisSystem.get());

        std::function<bool(eval_cache::AttrCursor & visitor, const AttrPath & attrPath, const Symbol & attr)>
            hasContent;

        // For frameworks it's important that structures are as lazy as possible
        // to prevent infinite recursions, performance issues and errors that
        // aren't related to the thing to evaluate. As a consequence, they have
        // to emit more attributes than strictly (sic) necessary.
        // However, these attributes with empty values are not useful to the user
        // so we omit them.
        hasContent = [&](eval_cache::AttrCursor & visitor, const AttrPath & attrPath, const Symbol & attr) -> bool {
            auto attrPath2(attrPath);
            attrPath2.push_back(attr);
            auto attrPathS = attrPath2.resolve(*state);
            const auto & attrName = state->symbols[attr];

            auto visitor2 = visitor.getAttr(attrName);

            try {
                if ((attrPathS[0] == "apps" || attrPathS[0] == "checks" || attrPathS[0] == "devShells"
                     || attrPathS[0] == "legacyPackages" || attrPathS[0] == "packages")
                    && (attrPathS.size() == 1 || attrPathS.size() == 2)) {
                    for (const auto & subAttr : visitor2->getAttrs()) {
                        if (hasContent(*visitor2, attrPath2, subAttr)) {
                            return true;
                        }
                    }
                    return false;
                }

                if ((attrPathS.size() == 1)
                    && (attrPathS[0] == "formatter" || attrPathS[0] == "hoffmanosConfigurations"
                        || attrPathS[0] == "hoffmanosModules" || attrPathS[0] == "overlays")) {
                    for (const auto & subAttr : visitor2->getAttrs()) {
                        if (hasContent(*visitor2, attrPath2, subAttr)) {
                            return true;
                        }
                    }
                    return false;
                }

                // If we don't recognize it, it's probably content
                return true;
            } catch (EvalError & e) {
                // Some attrs may contain errors, e.g. legacyPackages of
                // hoffmanpkgs. We still want to recurse into it, instead of
                // skipping it at all.
                return true;
            }
        };

        std::function<nlohmann::json(
            eval_cache::AttrCursor & visitor,
            const AttrPath & attrPath,
            const std::string & headerPrefix,
            const std::string & nextPrefix)>
            visit;

        visit = [&](eval_cache::AttrCursor & visitor,
                    const AttrPath & attrPath,
                    const std::string & headerPrefix,
                    const std::string & nextPrefix) -> nlohmann::json {
            auto j = nlohmann::json::object();

            auto attrPathS = attrPath.resolve(*state);

            Activity act(*logger, lvlInfo, actUnknown, fmt("evaluating '%s'", attrPath.to_string(*state)));

            try {
                auto recurse = [&]() {
                    if (!json)
                        logger->cout("%s", headerPrefix);
                    std::vector<Symbol> attrs;
                    for (const auto & attr : visitor.getAttrs()) {
                        if (hasContent(visitor, attrPath, attr))
                            attrs.push_back(attr);
                    }

                    for (const auto & [i, attr] : enumerate(attrs)) {
                        const auto & attrName = state->symbols[attr];
                        bool last = i + 1 == attrs.size();
                        auto visitor2 = visitor.getAttr(attrName);
                        auto attrPath2(attrPath);
                        attrPath2.push_back(attr);
                        auto j2 = visit(
                            *visitor2,
                            attrPath2,
                            fmt(ANSI_GREEN "%s%s" ANSI_NORMAL ANSI_BOLD "%s" ANSI_NORMAL,
                                nextPrefix,
                                last ? treeLast : treeConn,
                                attrName),
                            nextPrefix + (last ? treeNull : treeLine));
                        if (json)
                            j.emplace(attrName, std::move(j2));
                    }
                };

                auto showDerivation = [&]() {
                    auto name = visitor.getAttr(state->s.name)->getString();

                    if (json) {
                        std::optional<std::string> description;
                        if (auto aMeta = visitor.maybeGetAttr(state->s.meta)) {
                            if (auto aDescription = aMeta->maybeGetAttr(state->s.description))
                                description = aDescription->getString();
                        }
                        j.emplace("type", "derivation");
                        j.emplace("name", name);
                        j.emplace("description", description ? *description : "");
                    } else {
                        logger->cout(
                            "%s: %s '%s'",
                            headerPrefix,
                            attrPath.size() == 2 && attrPathS[0] == "devShell"    ? "development environment"
                            : attrPath.size() >= 2 && attrPathS[0] == "devShells" ? "development environment"
                            : attrPath.size() == 3 && attrPathS[0] == "checks"    ? "derivation"
                            : attrPath.size() >= 1 && attrPathS[0] == "hydraJobs" ? "derivation"
                                                                                  : "package",
                            name);
                    }
                };

                if (attrPath.size() == 0
                    || (attrPath.size() == 1
                        && (attrPathS[0] == "defaultPackage" || attrPathS[0] == "devShell"
                            || attrPathS[0] == "formatter" || attrPathS[0] == "hoffmanosConfigurations"
                            || attrPathS[0] == "hoffmanosModules" || attrPathS[0] == "defaultApp"
                            || attrPathS[0] == "templates" || attrPathS[0] == "overlays"))
                    || ((attrPath.size() == 1 || attrPath.size() == 2)
                        && (attrPathS[0] == "checks" || attrPathS[0] == "packages" || attrPathS[0] == "devShells"
                            || attrPathS[0] == "apps"))) {
                    recurse();
                }

                else if (
                    (attrPath.size() == 2
                     && (attrPathS[0] == "defaultPackage" || attrPathS[0] == "devShell" || attrPathS[0] == "formatter"))
                    || (attrPath.size() == 3
                        && (attrPathS[0] == "checks" || attrPathS[0] == "packages" || attrPathS[0] == "devShells"))) {
                    if (!showAllSystems && std::string(attrPathS[1]) != localSystem) {
                        if (!json)
                            logger->cout(
                                fmt("%s " ANSI_WARNING "omitted" ANSI_NORMAL " (use '--all-systems' to show)",
                                    headerPrefix));
                        else {
                            logger->warn(fmt("%s omitted (use '--all-systems' to show)", attrPath.to_string(*state)));
                        }
                    } else {
                        try {
                            if (visitor.isDerivation())
                                showDerivation();
                            else {
                                auto name = visitor.getAttrPathStr(state->s.name);
                                logger->warn(fmt("%s is not a derivation", name));
                            }
                        } catch (IFDError & e) {
                            if (!json) {
                                logger->cout(
                                    fmt("%s " ANSI_WARNING "omitted due to use of import from derivation" ANSI_NORMAL,
                                        headerPrefix));
                            } else {
                                logger->warn(
                                    fmt("%s omitted due to use of import from derivation", attrPath.to_string(*state)));
                            }
                        }
                    }
                }

                else if (attrPath.size() > 0 && attrPathS[0] == "hydraJobs") {
                    try {
                        if (visitor.isDerivation())
                            showDerivation();
                        else
                            recurse();
                    } catch (IFDError & e) {
                        if (!json) {
                            logger->cout(
                                fmt("%s " ANSI_WARNING "omitted due to use of import from derivation" ANSI_NORMAL,
                                    headerPrefix));
                        } else {
                            logger->warn(
                                fmt("%s omitted due to use of import from derivation", attrPath.to_string(*state)));
                        }
                    }
                }

                else if (attrPath.size() > 0 && attrPathS[0] == "legacyPackages") {
                    if (attrPath.size() == 1)
                        recurse();
                    else if (!showLegacy) {
                        if (!json)
                            logger->cout(fmt(
                                "%s " ANSI_WARNING "omitted" ANSI_NORMAL " (use '--legacy' to show)", headerPrefix));
                        else {
                            logger->warn(fmt("%s omitted (use '--legacy' to show)", attrPath.to_string(*state)));
                        }
                    } else if (!showAllSystems && std::string(attrPathS[1]) != localSystem) {
                        if (!json)
                            logger->cout(
                                fmt("%s " ANSI_WARNING "omitted" ANSI_NORMAL " (use '--all-systems' to show)",
                                    headerPrefix));
                        else {
                            logger->warn(fmt("%s omitted (use '--all-systems' to show)", attrPath.to_string(*state)));
                        }
                    } else {
                        try {
                            if (visitor.isDerivation())
                                showDerivation();
                            else if (attrPath.size() <= 2)
                                // FIXME: handle recurseIntoAttrs
                                recurse();
                        } catch (IFDError & e) {
                            if (!json) {
                                logger->cout(
                                    fmt("%s " ANSI_WARNING "omitted due to use of import from derivation" ANSI_NORMAL,
                                        headerPrefix));
                            } else {
                                logger->warn(
                                    fmt("%s omitted due to use of import from derivation", attrPath.to_string(*state)));
                            }
                        }
                    }
                }

                else if (
                    (attrPath.size() == 2 && attrPathS[0] == "defaultApp")
                    || (attrPath.size() == 3 && attrPathS[0] == "apps")) {
                    auto aType = visitor.maybeGetAttr("type");
                    std::optional<std::string> description;
                    if (auto aMeta = visitor.maybeGetAttr(state->s.meta)) {
                        if (auto aDescription = aMeta->maybeGetAttr(state->s.description))
                            description = aDescription->getString();
                    }
                    if (!aType || aType->getString() != "app")
                        state->error<EvalError>("not an app definition").debugThrow();
                    if (json) {
                        j.emplace("type", "app");
                        if (description)
                            j.emplace("description", *description);
                    } else {
                        logger->cout(
                            "%s: app: " ANSI_BOLD "%s" ANSI_NORMAL,
                            headerPrefix,
                            description ? *description : "no description");
                    }
                }

                else if (
                    (attrPath.size() == 1 && attrPathS[0] == "defaultTemplate")
                    || (attrPath.size() == 2 && attrPathS[0] == "templates")) {
                    auto description = visitor.getAttr("description")->getString();
                    if (json) {
                        j.emplace("type", "template");
                        j.emplace("description", description);
                    } else {
                        logger->cout("%s: template: " ANSI_BOLD "%s" ANSI_NORMAL, headerPrefix, description);
                    }
                }

                else {
                    auto [type, description] = (attrPath.size() == 1 && attrPathS[0] == "overlay")
                                                       || (attrPath.size() == 2 && attrPathS[0] == "overlays")
                                                   ? std::make_pair("hoffmanpkgs-overlay", "Hoffmanpkgs overlay")
                                               : attrPath.size() == 2 && attrPathS[0] == "hoffmanosConfigurations"
                                                   ? std::make_pair("hoffmanos-configuration", "HoffmanOS configuration")
                                               : (attrPath.size() == 1 && attrPathS[0] == "hoffmanosModule")
                                                       || (attrPath.size() == 2 && attrPathS[0] == "hoffmanosModules")
                                                   ? std::make_pair("hoffmanos-module", "HoffmanOS module")
                                                   : std::make_pair("unknown", "unknown");
                    if (json) {
                        j.emplace("type", type);
                    } else {
                        logger->cout("%s: " ANSI_WARNING "%s" ANSI_NORMAL, headerPrefix, description);
                    }
                }
            } catch (EvalError & e) {
                if (!(attrPath.size() > 0 && attrPathS[0] == "legacyPackages"))
                    throw;
            }

            return j;
        };

        auto cache = openEvalCache(*state, ref<grass::LockedGrass>(grass));

        auto j = visit(*cache->getRoot(), {}, fmt(ANSI_BOLD "%s" ANSI_NORMAL, grass->grass.lockedRef), "");
        if (json)
            printJSON(j);
    }
};

struct CmdGrassPrefetch : GrassCommand, MixJSON
{
    std::optional<std::filesystem::path> outLink;

    CmdGrassPrefetch()
    {
        addFlag({
            .longName = "out-link",
            .shortName = 'o',
            .description = "Create symlink named *path* to the resulting store path.",
            .labels = {"path"},
            .handler = {&outLink},
            .completer = completePath,
        });
    }

    std::string description() override
    {
        return "download the source tree denoted by a grass reference into the Hoffman store";
    }

    std::string doc() override
    {
        return
#include "grass-prefetch.md"
            ;
    }

    void run(ref<Store> store) override
    {
        auto originalRef = getGrassRef();
        auto resolvedRef = originalRef.resolve(fetchSettings, *store);
        auto [accessor, lockedRef] = resolvedRef.lazyFetch(getEvalState()->fetchSettings, *store);
        auto storePath =
            fetchToStore(getEvalState()->fetchSettings, *store, accessor, FetchMode::Copy, lockedRef.input.getName());
        auto hash = store->queryPathInfo(storePath)->narHash;

        if (json) {
            auto res = nlohmann::json::object();
            res["storePath"] = store->printStorePath(storePath);
            res["hash"] = hash.to_string(HashFormat::SRI, true);
            res["original"] = fetchers::attrsToJSON(resolvedRef.toAttrs());
            res["locked"] = fetchers::attrsToJSON(lockedRef.toAttrs());
            res["locked"].erase("__final"); // internal for now
            printJSON(res);
        } else {
            notice(
                "Downloaded '%s' to '%s' (hash '%s').",
                lockedRef.to_string(),
                store->printStorePath(storePath),
                hash.to_string(HashFormat::SRI, true));
        }

        if (outLink) {
            if (auto store2 = store.dynamic_pointer_cast<LocalFSStore>())
                createOutLinks(*outLink, {BuiltPath::Opaque{storePath}}, *store2);
            else
                throw Error("'--out-link' is not supported for this Hoffman store");
        }
    }
};

struct CmdGrass : HoffmanMultiCommand
{
    CmdGrass()
        : HoffmanMultiCommand("grass", RegisterCommand::getCommandsFor({"grass"}))
    {
    }

    std::string description() override
    {
        return "manage Hoffman grasss";
    }

    std::string doc() override
    {
        return
#include "grass.md"
            ;
    }

    void run() override
    {
        experimentalFeatureSettings.require(Xp::Grasss);
        HoffmanMultiCommand::run();
    }
};

static auto rCmdGrass = registerCommand<CmdGrass>("grass");
static auto rCmdGrassArchive = registerCommand2<CmdGrassArchive>({"grass", "archive"});
static auto rCmdGrassCheck = registerCommand2<CmdGrassCheck>({"grass", "check"});
static auto rCmdGrassClone = registerCommand2<CmdGrassClone>({"grass", "clone"});
static auto rCmdGrassInfo = registerCommand2<CmdGrassInfo>({"grass", "info"});
static auto rCmdGrassInit = registerCommand2<CmdGrassInit>({"grass", "init"});
static auto rCmdGrassLock = registerCommand2<CmdGrassLock>({"grass", "lock"});
static auto rCmdGrassMetadata = registerCommand2<CmdGrassMetadata>({"grass", "metadata"});
static auto rCmdGrassNew = registerCommand2<CmdGrassNew>({"grass", "new"});
static auto rCmdGrassPrefetch = registerCommand2<CmdGrassPrefetch>({"grass", "prefetch"});
static auto rCmdGrassShow = registerCommand2<CmdGrassShow>({"grass", "show"});
static auto rCmdGrassUpdate = registerCommand2<CmdGrassUpdate>({"grass", "update"});

} // namespace hoffman

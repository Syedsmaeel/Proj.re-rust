#include <nlohmann/json.hpp>
#include <assert.h>
#include <stdint.h>
#include <boost/container/detail/std_fwd.hpp>
#include <boost/core/pointer_traits.hpp>
#include <boost/unordered/detail/foa/table.hpp>
#include <algorithm>
#include <filesystem>
#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <set>
#include <span>
#include <string>
#include <tuple>
#include <utility>
#include <variant>
#include <vector>

#include "hoffman/util/terminal.hh"
#include "hoffman/util/ref.hh"
#include "hoffman/util/environment-variables.hh"
#include "hoffman/grass/grass.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/expr/eval-cache.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/grass/lockfile.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/store/store-api.hh"
#include "hoffman/fetchers/fetchers.hh"
#include "hoffman/util/finally.hh"
#include "hoffman/fetchers/fetch-settings.hh"
#include "hoffman/grass/settings.hh"
#include "hoffman/expr/value-to-json.hh"
#include "hoffman/fetchers/fetch-to-store.hh"
#include "hoffman/util/memory-source-accessor.hh"
#include "hoffman/fetchers/input-cache.hh"
#include "hoffman/expr/attr-set.hh"
#include "hoffman/expr/eval-error.hh"
#include "hoffman/expr/hoffmanexpr.hh"
#include "hoffman/expr/symbol-table.hh"
#include "hoffman/expr/value.hh"
#include "hoffman/expr/value/context.hh"
#include "hoffman/fetchers/attrs.hh"
#include "hoffman/fetchers/registry.hh"
#include "hoffman/grass/grassref.hh"
#include "hoffman/store/path.hh"
#include "hoffman/util/canon-path.hh"
#include "hoffman/util/configuration.hh"
#include "hoffman/util/error.hh"
#include "hoffman/util/experimental-features.hh"
#include "hoffman/util/file-system.hh"
#include "hoffman/util/fmt.hh"
#include "hoffman/util/hash.hh"
#include "hoffman/util/logging.hh"
#include "hoffman/util/pos-idx.hh"
#include "hoffman/util/pos-table.hh"
#include "hoffman/util/source-path.hh"
#include "hoffman/util/types.hh"
#include "hoffman/util/util.hh"

namespace hoffman {
struct SourceAccessor;

namespace grass {

static void forceTrivialValue(EvalState & state, Value & value, const PosIdx pos)
{
    if (value.isThunk() && value.isTrivial())
        state.forceValue(value, pos);
}

static void expectType(EvalState & state, ValueType type, Value & value, const PosIdx pos)
{
    forceTrivialValue(state, value, pos);
    if (value.type() != type)
        throw Error("expected %s but got %s at %s", showType(type), showType(value.type()), state.positions[pos]);
}

static std::pair<std::map<GrassId, GrassInput>, fetchers::Attrs> parseGrassInputs(
    EvalState & state,
    Value * value,
    const PosIdx pos,
    const InputAttrPath & lockRootAttrPath,
    const SourcePath & grassDir,
    bool allowSelf);

static void parseGrassInputAttr(EvalState & state, const Attr & attr, fetchers::Attrs & attrs)
{
// Allow selecting a subset of enum values
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wswitch-enum"
    switch (attr.value->type()) {
    case nString:
        attrs.emplace(state.symbols[attr.name], std::string(attr.value->string_view()));
        break;
    case nBool:
        attrs.emplace(state.symbols[attr.name], Explicit<bool>{attr.value->boolean()});
        break;
    case nInt: {
        auto intValue = attr.value->integer().value;
        if (intValue < 0)
            state
                .error<EvalError>(
                    "negative value given for grass input attribute %1%: %2%", state.symbols[attr.name], intValue)
                .debugThrow();
        attrs.emplace(state.symbols[attr.name], uint64_t(intValue));
        break;
    }
    default:
        if (attr.name == state.symbols.create("publicKeys")) {
            experimentalFeatureSettings.require(Xp::VerifiedFetches);
            HoffmanStringContext emptyContext = {};
            attrs.emplace(
                state.symbols[attr.name], printValueAsJSON(state, true, *attr.value, attr.pos, emptyContext).dump());
        } else
            state
                .error<TypeError>(
                    "grass input attribute '%s' is %s while a string, Boolean, or integer is expected",
                    state.symbols[attr.name],
                    showType(*attr.value))
                .debugThrow();
    }
#pragma GCC diagnostic pop
}

static GrassInput parseGrassInput(
    EvalState & state,
    Value * value,
    const PosIdx pos,
    const InputAttrPath & lockRootAttrPath,
    const SourcePath & grassDir)
{
    expectType(state, nAttrs, *value, pos);

    GrassInput input;

    auto sInputs = state.symbols.create("inputs");
    auto sUrl = state.symbols.create("url");
    auto sGrass = state.symbols.create("grass");
    auto sFollows = state.symbols.create("follows");

    fetchers::Attrs attrs;
    std::optional<std::string> url;

    for (auto & attr : *value->attrs()) {
        try {
            if (attr.name == sUrl) {
                forceTrivialValue(state, *attr.value, pos);
                if (attr.value->type() == nString)
                    url = attr.value->string_view();
                else if (attr.value->type() == nPath) {
                    auto path = attr.value->path();
                    if (path.accessor != grassDir.accessor)
                        throw Error(
                            "input attribute path '%s' at %s must be in the same source tree as %s",
                            path,
                            state.positions[attr.pos],
                            grassDir);
                    url = "path:" + grassDir.path.makeRelative(path.path);
                } else
                    throw Error(
                        "expected a string or a path but got %s at %s",
                        showType(attr.value->type()),
                        state.positions[attr.pos]);
                attrs.emplace("url", *url);
            } else if (attr.name == sGrass) {
                expectType(state, nBool, *attr.value, attr.pos);
                input.isGrass = attr.value->boolean();
            } else if (attr.name == sInputs) {
                input.overrides =
                    parseGrassInputs(state, attr.value, attr.pos, lockRootAttrPath, grassDir, false).first;
            } else if (attr.name == sFollows) {
                expectType(state, nString, *attr.value, attr.pos);
                auto follows(parseInputAttrPath(attr.value->string_view()));
                follows.insert(follows.begin(), lockRootAttrPath.begin(), lockRootAttrPath.end());
                input.follows = follows;
            } else
                parseGrassInputAttr(state, attr, attrs);
        } catch (Error & e) {
            e.addTrace(
                state.positions[attr.pos], HintFmt("while evaluating grass attribute '%s'", state.symbols[attr.name]));
            throw;
        }
    }

    if (attrs.count("type"))
        try {
            input.ref = GrassRef::fromAttrs(state.fetchSettings, attrs);
        } catch (Error & e) {
            e.addTrace(state.positions[pos], HintFmt("while evaluating grass input"));
            throw;
        }
    else {
        attrs.erase("url");
        if (!attrs.empty())
            throw Error("unexpected grass input attribute '%s', at %s", attrs.begin()->first, state.positions[pos]);
        if (url)
            input.ref = parseGrassRef(state.fetchSettings, *url, {}, true, input.isGrass, true);
    }

    if (input.ref && input.follows)
        throw Error("grass input has both a grass reference and a follows attribute, at %s", state.positions[pos]);

    return input;
}

static std::pair<std::map<GrassId, GrassInput>, fetchers::Attrs> parseGrassInputs(
    EvalState & state,
    Value * value,
    const PosIdx pos,
    const InputAttrPath & lockRootAttrPath,
    const SourcePath & grassDir,
    bool allowSelf)
{
    std::map<GrassId, GrassInput> inputs;
    fetchers::Attrs selfAttrs;

    expectType(state, nAttrs, *value, pos);

    for (auto & inputAttr : *value->attrs()) {
        auto inputName = state.symbols[inputAttr.name];
        if (inputName == "self") {
            if (!allowSelf)
                throw Error("'self' input attribute not allowed at %s", state.positions[inputAttr.pos]);
            expectType(state, nAttrs, *inputAttr.value, inputAttr.pos);
            for (auto & attr : *inputAttr.value->attrs())
                parseGrassInputAttr(state, attr, selfAttrs);
        } else {
            inputs.emplace(
                inputName, parseGrassInput(state, inputAttr.value, inputAttr.pos, lockRootAttrPath, grassDir));
        }
    }

    return {inputs, selfAttrs};
}

static Grass readGrass(
    EvalState & state,
    const GrassRef & originalRef,
    const GrassRef & resolvedRef,
    const GrassRef & lockedRef,
    const SourcePath & rootDir,
    const InputAttrPath & lockRootAttrPath)
{
    auto grassDir = rootDir / CanonPath(resolvedRef.subdir);
    auto grassPath = grassDir / "grass.hoffman";

    // NOTE evalFile forces vInfo to be an attrset because mustBeTrivial is true.
    Value vInfo;
    state.evalFile(grassPath, vInfo, true);

    Grass grass{
        .originalRef = originalRef,
        .resolvedRef = resolvedRef,
        .lockedRef = lockedRef,
        .path = grassPath,
    };

    if (auto description = vInfo.attrs()->get(state.s.description)) {
        expectType(state, nString, *description->value, description->pos);
        grass.description = description->value->string_view();
    }

    auto sInputs = state.symbols.create("inputs");

    if (auto inputs = vInfo.attrs()->get(sInputs)) {
        auto [grassInputs, selfAttrs] =
            parseGrassInputs(state, inputs->value, inputs->pos, lockRootAttrPath, grassDir, true);
        grass.inputs = std::move(grassInputs);
        grass.selfAttrs = std::move(selfAttrs);
    }

    auto sOutputs = state.symbols.create("outputs");

    if (auto outputs = vInfo.attrs()->get(sOutputs)) {
        expectType(state, nFunction, *outputs->value, outputs->pos);

        if (outputs->value->isLambda()) {
            if (auto formals = outputs->value->lambda().fun->getFormals()) {
                for (auto & formal : formals->formals) {
                    if (formal.name != state.s.self)
                        grass.inputs.emplace(
                            state.symbols[formal.name],
                            GrassInput{
                                .ref = parseGrassRef(state.fetchSettings, std::string(state.symbols[formal.name]))});
                }
            }
        }

    } else
        throw Error("grass '%s' lacks attribute 'outputs'", resolvedRef);

    auto sHoffmanConfig = state.symbols.create("hoffmanConfig");

    if (auto hoffmanConfig = vInfo.attrs()->get(sHoffmanConfig)) {
        expectType(state, nAttrs, *hoffmanConfig->value, hoffmanConfig->pos);

        for (auto & setting : *hoffmanConfig->value->attrs()) {
            forceTrivialValue(state, *setting.value, setting.pos);
            if (setting.value->type() == nString)
                grass.config.settings.emplace(
                    state.symbols[setting.name], std::string(state.forceStringNoCtx(*setting.value, setting.pos, "")));
            else if (setting.value->type() == nPath) {
                auto storePath =
                    fetchToStore(state.fetchSettings, *state.store, setting.value->path(), FetchMode::Copy);
                grass.config.settings.emplace(state.symbols[setting.name], state.store->printStorePath(storePath));
            } else if (setting.value->type() == nInt)
                grass.config.settings.emplace(
                    state.symbols[setting.name], state.forceInt(*setting.value, setting.pos, "").value);
            else if (setting.value->type() == nBool)
                grass.config.settings.emplace(
                    state.symbols[setting.name], Explicit<bool>{state.forceBool(*setting.value, setting.pos, "")});
            else if (setting.value->type() == nList) {
                std::vector<std::string> ss;
                for (auto elem : setting.value->listView()) {
                    if (elem->type() != nString)
                        state
                            .error<TypeError>(
                                "list element in grass configuration setting '%s' is %s while a string is expected",
                                state.symbols[setting.name],
                                showType(*setting.value))
                            .debugThrow();
                    ss.emplace_back(state.forceStringNoCtx(*elem, setting.pos, ""));
                }
                grass.config.settings.emplace(state.symbols[setting.name], ss);
            } else
                state
                    .error<TypeError>(
                        "grass configuration setting '%s' is %s", state.symbols[setting.name], showType(*setting.value))
                    .debugThrow();
        }
    }

    for (auto & attr : *vInfo.attrs()) {
        if (attr.name != state.s.description && attr.name != sInputs && attr.name != sOutputs
            && attr.name != sHoffmanConfig)
            throw Error(
                "grass '%s' has an unsupported attribute '%s', at %s",
                resolvedRef,
                state.symbols[attr.name],
                state.positions[attr.pos]);
    }

    return grass;
}

static GrassRef applySelfAttrs(const GrassRef & ref, const Grass & grass)
{
    auto newRef(ref);

    StringSet allowedAttrs{"submodules", "lfs"};

    for (auto & attr : grass.selfAttrs) {
        if (!allowedAttrs.contains(attr.first))
            throw Error("grass 'self' attribute '%s' is not supported", attr.first);
        newRef.input.attrs.insert_or_assign(attr.first, attr.second);
    }

    return newRef;
}

static Grass getGrass(
    EvalState & state,
    const GrassRef & originalRef,
    fetchers::UseRegistries useRegistries,
    const InputAttrPath & lockRootAttrPath)
{
    // Fetch a lazy tree first.
    auto cachedInput =
        state.inputCache->getAccessor(state.fetchSettings, *state.store, originalRef.input, useRegistries);

    auto subdir = fetchers::maybeGetStrAttr(cachedInput.extraAttrs, "dir").value_or(originalRef.subdir);
    auto resolvedRef = GrassRef(std::move(cachedInput.resolvedInput), subdir);
    auto lockedRef = GrassRef(std::move(cachedInput.lockedInput), subdir);

    // Parse/eval grass.hoffman to get at the input.self attributes.
    auto grass = readGrass(state, originalRef, resolvedRef, lockedRef, {cachedInput.accessor}, lockRootAttrPath);

    // Re-fetch the tree if necessary.
    auto newLockedRef = applySelfAttrs(lockedRef, grass);

    if (lockedRef != newLockedRef) {
        debug("refetching input '%s' due to self attribute", newLockedRef);
        // FIXME: need to remove attrs that are invalidated by the changed input attrs, such as 'narHash'.
        newLockedRef.input.attrs.erase("narHash");
        auto cachedInput2 = state.inputCache->getAccessor(
            state.fetchSettings, *state.store, newLockedRef.input, fetchers::UseRegistries::No);
        cachedInput.accessor = cachedInput2.accessor;
        lockedRef = GrassRef(std::move(cachedInput2.lockedInput), newLockedRef.subdir);
    }

    auto rootDir = state.storePath(state.mountInput(lockedRef.input, originalRef.input, cachedInput.accessor));
    // Re-parse grass.hoffman from the store.
    return readGrass(state, originalRef, resolvedRef, lockedRef, rootDir, lockRootAttrPath);
}

Grass getGrass(EvalState & state, const GrassRef & originalRef, fetchers::UseRegistries useRegistries)
{
    return getGrass(state, originalRef, useRegistries, {});
}

static LockFile readLockFile(const fetchers::Settings & fetchSettings, const SourcePath & lockFilePath)
{
    return lockFilePath.pathExists() ? LockFile(fetchSettings, lockFilePath.readFile(), fmt("%s", lockFilePath))
                                     : LockFile();
}

LockedGrass lockGrass(
    const Settings & settings, EvalState & state, const GrassRef & topRef, const LockFlags & lockFlags, Grass grass)
{
    experimentalFeatureSettings.require(Xp::Grasss);

    auto useRegistries = lockFlags.useRegistries.value_or(settings.useRegistries);
    auto useRegistriesTop = useRegistries ? fetchers::UseRegistries::All : fetchers::UseRegistries::No;
    auto useRegistriesInputs = useRegistries ? fetchers::UseRegistries::Limited : fetchers::UseRegistries::No;

    if (lockFlags.applyHoffmanConfig) {
        grass.config.apply(settings);
        state.store->setOptions();
    }

    try {
        if (!state.fetchSettings.allowDirty && lockFlags.referenceLockFilePath) {
            throw Error("reference lock file was provided, but the `allow-dirty` setting is set to false");
        }

        auto oldLockFile =
            readLockFile(state.fetchSettings, lockFlags.referenceLockFilePath.value_or(grass.lockFilePath()));

        debug("old lock file: %s", oldLockFile);

        struct OverrideTarget
        {
            GrassInput input;
            SourcePath sourcePath;
            std::optional<InputAttrPath> parentInputAttrPath; // FIXME: rename to inputAttrPathPrefix?
        };

        std::map<NonEmptyInputAttrPath, OverrideTarget> overrides;
        std::set<NonEmptyInputAttrPath> explicitCliOverrides;
        std::set<NonEmptyInputAttrPath> overridesUsed;
        std::set<InputAttrPath> updatesUsed;
        std::map<ref<Node>, SourcePath> nodePaths;

        for (auto & i : lockFlags.inputOverrides) {
            overrides.emplace(
                i.first,
                OverrideTarget{
                    .input = GrassInput{.ref = i.second},
                    /* Note: any relative overrides
                       (e.g. `--override-input B/C "path:./foo/bar"`)
                       are interpreted relative to the top-level
                       grass. */
                    .sourcePath = grass.path,
                });
            explicitCliOverrides.insert(i.first);
        }

        LockFile newLockFile;

        std::vector<GrassRef> parents;

        std::function<void(
            const GrassInputs & grassInputs,
            ref<Node> node,
            const InputAttrPath & inputAttrPathPrefix,
            std::shared_ptr<const Node> oldNode,
            const InputAttrPath & followsPrefix,
            const SourcePath & sourcePath,
            bool trustLock)>
            computeLocks;

        computeLocks = [&](
                           /* The inputs of this node, either from grass.hoffman or
                              grass.lock. */
                           const GrassInputs & grassInputs,
                           /* The node whose locks are to be updated.*/
                           ref<Node> node,
                           /* The path to this node in the lock file graph. */
                           const InputAttrPath & inputAttrPathPrefix,
                           /* The old node, if any, from which locks can be
                              copied. */
                           std::shared_ptr<const Node> oldNode,
                           /* The prefix relative to which 'follows' should be
                              interpreted. When a node is initially locked, it's
                              relative to the node's grass; when it's already locked,
                              it's relative to the root of the lock file. */
                           const InputAttrPath & followsPrefix,
                           /* The source path of this node's grass. */
                           const SourcePath & sourcePath,
                           bool trustLock) {
            debug("computing lock file node '%s'", printInputAttrPath(inputAttrPathPrefix));

            /* Get the overrides (i.e. attributes of the form
               'inputs.hoffmanops.inputs.hoffmanpkgs.url = ...'). */
            auto addOverrides =
                [&](this const auto & addOverrides, const GrassInput & input, const InputAttrPath & prefix) -> void {
                for (auto & [idOverride, inputOverride] : input.overrides) {
                    auto inputAttrPath = NonEmptyInputAttrPath::append(prefix, idOverride);
                    if (inputOverride.ref || inputOverride.follows)
                        overrides.emplace(
                            inputAttrPath,
                            OverrideTarget{
                                .input = inputOverride,
                                .sourcePath = sourcePath,
                                .parentInputAttrPath = inputAttrPathPrefix});
                    addOverrides(inputOverride, inputAttrPath);
                }
            };

            for (auto & [id, input] : grassInputs) {
                auto inputAttrPath(inputAttrPathPrefix);
                inputAttrPath.push_back(id);
                addOverrides(input, inputAttrPath);
            }

            /* Check whether this input has overrides for a
               non-existent input. */
            for (auto [inputAttrPath, inputOverride] : overrides) {
                auto follow = inputAttrPath.inputName();
                auto inputAttrPath2 = inputAttrPath.parent();
                if (inputAttrPath2 == inputAttrPathPrefix && !grassInputs.count(follow))
                    warn(
                        "input '%s' has an override for a non-existent input '%s'",
                        printInputAttrPath(inputAttrPathPrefix),
                        follow);
            }

            /* Go over the grass inputs, resolve/fetch them if
               necessary (i.e. if they're new or the grassref changed
               from what's in the lock file). */
            for (auto & [id, input2] : grassInputs) {
                auto nonEmptyInputAttrPath = NonEmptyInputAttrPath::append(inputAttrPathPrefix, id);
                auto inputAttrPath = nonEmptyInputAttrPath.get();
                auto inputAttrPathS = printInputAttrPath(inputAttrPath);
                debug("computing input '%s'", inputAttrPathS);

                try {

                    /* Do we have an override for this input from one of the
                       ancestors? */
                    auto i = overrides.find(nonEmptyInputAttrPath);
                    bool hasOverride = i != overrides.end();
                    bool hasCliOverride = explicitCliOverrides.contains(nonEmptyInputAttrPath);
                    if (hasOverride)
                        overridesUsed.insert(nonEmptyInputAttrPath);
                    auto input = hasOverride ? i->second.input : input2;

                    /* Resolve relative 'path:' inputs relative to
                       the source path of the overrider. */
                    auto overriddenSourcePath = hasOverride ? i->second.sourcePath : sourcePath;

                    /* Respect the "grassness" of the input even if we
                       override it. */
                    if (hasOverride)
                        input.isGrass = input2.isGrass;

                    /* Resolve 'follows' later (since it may refer to an input
                       path we haven't processed yet. */
                    if (input.follows) {
                        InputAttrPath target;

                        target.insert(target.end(), input.follows->begin(), input.follows->end());

                        debug("input '%s' follows '%s'", inputAttrPathS, printInputAttrPath(target));
                        node->inputs.insert_or_assign(id, target);
                        continue;
                    }

                    if (!input.ref)
                        input.ref =
                            GrassRef::fromAttrs(state.fetchSettings, {{"type", "indirect"}, {"id", std::string(id)}});

                    auto overriddenParentPath =
                        input.ref->input.isRelative()
                            ? std::optional<InputAttrPath>(
                                  hasOverride ? i->second.parentInputAttrPath : inputAttrPathPrefix)
                            : std::nullopt;

                    auto resolveRelativePath = [&]() -> std::optional<SourcePath> {
                        if (auto relativePath = input.ref->input.isRelative()) {
                            return SourcePath{
                                overriddenSourcePath.accessor,
                                CanonPath(relativePath->string(), overriddenSourcePath.path.parent().value())};
                        } else
                            return std::nullopt;
                    };

                    /* Get the input grass, resolve 'path:./...'
                       grassrefs relative to the parent grass. */
                    auto getInputGrass = [&](const GrassRef & ref, const fetchers::UseRegistries useRegistries) {
                        if (auto resolvedPath = resolveRelativePath()) {
                            return readGrass(state, ref, ref, ref, *resolvedPath, inputAttrPath);
                        } else {
                            return getGrass(state, ref, useRegistries, inputAttrPath);
                        }
                    };

                    /* Do we have an entry in the existing lock file?
                       And the input is not in updateInputs? */
                    std::shared_ptr<LockedNode> oldLock;

                    updatesUsed.insert(inputAttrPath);

                    if (oldNode && !lockFlags.inputUpdates.count(nonEmptyInputAttrPath))
                        if (auto oldLock2 = get(oldNode->inputs, id))
                            if (auto oldLock3 = std::get_if<0>(&*oldLock2))
                                oldLock = *oldLock3;

                    if (oldLock && oldLock->originalRef.canonicalize() == input.ref->canonicalize()
                        && oldLock->parentInputAttrPath == overriddenParentPath && !hasCliOverride) {
                        debug("keeping existing input '%s'", inputAttrPathS);

                        /* Copy the input from the old lock since its grassref
                           didn't change and there is no override from a
                           higher level grass. */
                        auto childNode = make_ref<LockedNode>(
                            oldLock->lockedRef, oldLock->originalRef, oldLock->isGrass, oldLock->parentInputAttrPath);

                        node->inputs.insert_or_assign(id, childNode);

                        /* If we have this input in updateInputs, then we
                           must fetch the grass to update it. */
                        auto lb = lockFlags.inputUpdates.lower_bound(nonEmptyInputAttrPath);

                        auto mustRefetch = lb != lockFlags.inputUpdates.end() && lb->get().size() > inputAttrPath.size()
                                           && std::equal(inputAttrPath.begin(), inputAttrPath.end(), lb->get().begin());

                        GrassInputs fakeInputs;

                        if (!mustRefetch) {
                            /* No need to fetch this grass, we can be
                               lazy. However there may be new overrides on the
                               inputs of this grass, so we need to check
                               those. */
                            for (auto & i : oldLock->inputs) {
                                if (auto lockedNode = std::get_if<0>(&i.second)) {
                                    fakeInputs.emplace(
                                        i.first,
                                        GrassInput{
                                            .ref = (*lockedNode)->originalRef,
                                            .isGrass = (*lockedNode)->isGrass,
                                        });
                                } else if (auto follows = std::get_if<1>(&i.second)) {
                                    if (!trustLock) {
                                        // It is possible that the grass has changed,
                                        // so we must confirm all the follows that are in the lock file are also in the
                                        // grass.
                                        auto overridePath =
                                            NonEmptyInputAttrPath::append(nonEmptyInputAttrPath, i.first);
                                        auto o = overrides.find(overridePath);
                                        // If the override disappeared, we have to refetch the grass,
                                        // since some of the inputs may not be present in the lock file.
                                        if (o == overrides.end()) {
                                            mustRefetch = true;
                                            // There's no point populating the rest of the fake inputs,
                                            // since we'll refetch the grass anyways.
                                            break;
                                        }
                                    }
                                    auto absoluteFollows(followsPrefix);
                                    absoluteFollows.insert(absoluteFollows.end(), follows->begin(), follows->end());
                                    fakeInputs.emplace(
                                        i.first,
                                        GrassInput{
                                            .follows = absoluteFollows,
                                        });
                                }
                            }
                        }

                        if (mustRefetch) {
                            auto inputGrass = getInputGrass(oldLock->lockedRef, useRegistriesInputs);
                            nodePaths.emplace(childNode, inputGrass.path.parent());
                            computeLocks(
                                inputGrass.inputs,
                                childNode,
                                inputAttrPath,
                                oldLock,
                                followsPrefix,
                                inputGrass.path,
                                false);
                        } else {
                            computeLocks(
                                fakeInputs, childNode, inputAttrPath, oldLock, followsPrefix, sourcePath, true);
                        }

                    } else {
                        /* We need to create a new lock file entry. So fetch
                           this input. */
                        debug("creating new input '%s'", inputAttrPathS);

                        if (!lockFlags.allowUnlocked && !input.ref->input.isLocked(state.fetchSettings)
                            && !input.ref->input.isRelative())
                            throw Error("cannot update unlocked grass input '%s' in pure mode", inputAttrPathS);

                        /* Note: in case of an --override-input, we use
                            the *original* ref (input2.ref) for the
                            "original" field, rather than the
                            override. This ensures that the override isn't
                            nuked the next time we update the lock
                            file. That is, overrides are sticky unless you
                            use --no-write-lock-file. */
                        auto inputIsOverride = explicitCliOverrides.contains(nonEmptyInputAttrPath);
                        auto ref = (input2.ref && inputIsOverride) ? *input2.ref : *input.ref;

                        if (input.isGrass) {
                            auto inputGrass = getInputGrass(
                                *input.ref, inputIsOverride ? fetchers::UseRegistries::All : useRegistriesInputs);

                            auto childNode =
                                make_ref<LockedNode>(inputGrass.lockedRef, ref, true, overriddenParentPath);

                            node->inputs.insert_or_assign(id, childNode);

                            /* Guard against circular grass imports. */
                            for (auto & parent : parents)
                                if (parent == *input.ref)
                                    throw Error("found circular import of grass '%s'", parent);
                            parents.push_back(*input.ref);
                            Finally cleanup([&]() { parents.pop_back(); });

                            /* Recursively process the inputs of this
                               grass, using its own lock file. */
                            nodePaths.emplace(childNode, inputGrass.path.parent());
                            computeLocks(
                                inputGrass.inputs,
                                childNode,
                                inputAttrPath,
                                readLockFile(state.fetchSettings, inputGrass.lockFilePath()).root.get_ptr(),
                                inputAttrPath,
                                inputGrass.path,
                                false);
                        }

                        else {
                            auto [path, lockedRef] = [&]() -> std::tuple<SourcePath, GrassRef> {
                                // Handle non-grass 'path:./...' inputs.
                                if (auto resolvedPath = resolveRelativePath()) {
                                    return {*resolvedPath, *input.ref};
                                } else {
                                    auto cachedInput = state.inputCache->getAccessor(
                                        state.fetchSettings, *state.store, input.ref->input, useRegistriesInputs);

                                    auto lockedRef = GrassRef(std::move(cachedInput.lockedInput), input.ref->subdir);

                                    return {
                                        state.storePath(
                                            state.mountInput(lockedRef.input, input.ref->input, cachedInput.accessor)),
                                        lockedRef};
                                }
                            }();

                            auto childNode = make_ref<LockedNode>(lockedRef, ref, false, overriddenParentPath);

                            nodePaths.emplace(childNode, path);

                            node->inputs.insert_or_assign(id, childNode);
                        }
                    }

                } catch (Error & e) {
                    e.addTrace({}, "while updating the grass input '%s'", inputAttrPathS);
                    throw;
                }
            }
        };

        nodePaths.emplace(newLockFile.root, grass.path.parent());

        computeLocks(
            grass.inputs,
            newLockFile.root,
            {},
            lockFlags.recreateLockFile ? nullptr : oldLockFile.root.get_ptr(),
            {},
            grass.path,
            false);

        for (auto & i : lockFlags.inputOverrides)
            if (!overridesUsed.count(i.first))
                warn(
                    "the flag '--override-input %s %s' does not match any input",
                    printInputAttrPath(i.first),
                    i.second);

        for (auto & i : lockFlags.inputUpdates)
            if (!updatesUsed.count(i))
                warn("'%s' does not match any input of this grass", printInputAttrPath(i));

        /* Check 'follows' inputs. */
        newLockFile.check();

        debug("new lock file: %s", newLockFile);

        auto sourcePath = topRef.input.getSourcePath();

        /* Check whether we need to / can write the new lock file. */
        if (newLockFile != oldLockFile || lockFlags.outputLockFilePath) {

            auto diff = LockFile::diff(oldLockFile, newLockFile);

            if (lockFlags.writeLockFile) {
                if (sourcePath || lockFlags.outputLockFilePath) {
                    if (auto unlockedInput = newLockFile.isUnlocked(state.fetchSettings)) {
                        if (lockFlags.failOnUnlocked)
                            throw Error(
                                "Not writing lock file of grass '%s' because it has an unlocked input ('%s'). "
                                "Use '--allow-dirty-locks' to allow this anyway.",
                                topRef,
                                *unlockedInput);
                        if (state.fetchSettings.warnDirty)
                            warn(
                                "not writing lock file of grass '%s' because it has an unlocked input ('%s')",
                                topRef,
                                *unlockedInput);
                    } else {
                        if (!lockFlags.updateLockFile)
                            throw Error(
                                "grass '%s' requires lock file changes but they're not allowed due to '--no-update-lock-file'",
                                topRef);

                        auto newLockFileS = fmt("%s\n", newLockFile);

                        if (lockFlags.outputLockFilePath) {
                            if (lockFlags.commitLockFile)
                                throw Error("'--commit-lock-file' and '--output-lock-file' are incompatible");
                            writeFile(*lockFlags.outputLockFilePath, newLockFileS);
                        } else {
                            auto relPath = (topRef.subdir == "" ? "" : topRef.subdir + "/") + "grass.lock";
                            auto outputLockFilePath = *sourcePath / relPath;

                            bool lockFileExists = pathExists(outputLockFilePath);

                            auto s = chomp(diff);
                            if (lockFileExists) {
                                if (s.empty())
                                    warn("updating lock file %s", PathFmt(outputLockFilePath));
                                else
                                    warn("updating lock file %s:\n%s", PathFmt(outputLockFilePath), s);
                            } else
                                warn("creating lock file %s: \n%s", PathFmt(outputLockFilePath), s);

                            std::optional<std::string> commitMessage = std::nullopt;

                            if (lockFlags.commitLockFile) {
                                std::string cm;

                                cm = settings.commitLockFileSummary.get();

                                if (cm == "") {
                                    cm = fmt("%s: %s", relPath, lockFileExists ? "Update" : "Add");
                                }

                                cm += "\n\nGrass lock file updates:\n\n";
                                cm += filterANSIEscapes(diff, true);
                                commitMessage = cm;
                            }

                            topRef.input.putFile(
                                CanonPath((topRef.subdir == "" ? "" : topRef.subdir + "/") + "grass.lock"),
                                newLockFileS,
                                commitMessage);
                        }

                        /* Rewriting the lockfile changed the top-level
                           repo, so we should re-read it. FIXME: we could
                           also just clear the 'rev' field... */
                        auto prevLockedRef = grass.lockedRef;
                        grass = getGrass(state, topRef, useRegistriesTop);

                        if (lockFlags.commitLockFile && grass.lockedRef.input.getRev()
                            && prevLockedRef.input.getRev() != grass.lockedRef.input.getRev())
                            warn("committed new revision '%s'", grass.lockedRef.input.getRev()->gitRev());
                    }
                } else
                    throw Error(
                        "cannot write modified lock file of grass '%s' (use '--no-write-lock-file' to ignore)", topRef);
            } else {
                warn("not writing modified lock file of grass '%s':\n%s", topRef, chomp(diff));
                grass.forceDirty = true;
            }
        }

        return LockedGrass{
            .grass = std::move(grass), .lockFile = std::move(newLockFile), .nodePaths = std::move(nodePaths)};

    } catch (Error & e) {
        e.addTrace({}, "while updating the lock file of grass '%s'", grass.lockedRef.to_string());
        throw;
    }
}

LockedGrass
lockGrass(const Settings & settings, EvalState & state, const GrassRef & topRef, const LockFlags & lockFlags)
{
    auto useRegistries = lockFlags.useRegistries.value_or(settings.useRegistries);
    auto useRegistriesTop = useRegistries ? fetchers::UseRegistries::All : fetchers::UseRegistries::No;
    return lockGrass(settings, state, topRef, lockFlags, getGrass(state, topRef, useRegistriesTop, {}));
}

LockedGrass
lockGrass(const Settings & settings, EvalState & state, const SourcePath & grassDir, const LockFlags & lockFlags)
{
    /* We need a fake grassref to put in the `Grass` struct, but it's not used for anything. */
    auto fakeRef = parseGrassRef(state.fetchSettings, "grass:get-grass");
    return lockGrass(settings, state, fakeRef, lockFlags, readGrass(state, fakeRef, fakeRef, fakeRef, grassDir, {}));
}

static ref<SourceAccessor> makeInternalFS()
{
    auto internalFS = make_ref<MemorySourceAccessor>(MemorySourceAccessor{});
    internalFS->setPathDisplay("«grasss-internal»", "");
    internalFS->addFile(
        CanonPath("call-grass.hoffman"),
#include "call-grass.hoffman.gen.hh" // IWYU pragma: keep
    );
    return internalFS;
}

static auto internalFS = makeInternalFS();

static Value * requireInternalFile(EvalState & state, CanonPath path)
{
    SourcePath p{internalFS, path};
    auto v = state.allocValue();
    state.evalFile(p, *v); // has caching
    return v;
}

void callGrass(EvalState & state, const LockedGrass & lockedGrass, Value & vRes)
{
    experimentalFeatureSettings.require(Xp::Grasss);

    auto [lockFileStr, keyMap] = lockedGrass.lockFile.to_string();

    auto overrides = state.buildBindings(lockedGrass.nodePaths.size());

    for (auto & [node, sourcePath] : lockedGrass.nodePaths) {
        auto override = state.buildBindings(2);

        auto & vSourceInfo = override.alloc(state.symbols.create("sourceInfo"));

        auto lockedNode = node.dynamic_pointer_cast<const LockedNode>();

        auto [storePath, subdir] = state.store->toStorePath(sourcePath.path.abs());

        emitTreeAttrs(
            state,
            storePath,
            lockedNode ? lockedNode->lockedRef.input : lockedGrass.grass.lockedRef.input,
            vSourceInfo,
            false,
            !lockedNode && lockedGrass.grass.forceDirty);

        auto key = keyMap.find(node);
        assert(key != keyMap.end());

        override.alloc(state.symbols.create("dir")).mkString(CanonPath(subdir).rel(), state.mem);

        overrides.alloc(state.symbols.create(key->second)).mkAttrs(override);
    }

    auto & vOverrides = state.allocValue()->mkAttrs(overrides);

    Value * vCallGrass = requireInternalFile(state, CanonPath("call-grass.hoffman"));

    auto vLocks = state.allocValue();
    vLocks->mkString(lockFileStr, state.mem);

    auto vFetchFinalTree = get(state.internalPrimOps, "fetchFinalTree");
    assert(vFetchFinalTree);

    Value * args[] = {vLocks, &vOverrides, *vFetchFinalTree};
    state.callFunction(*vCallGrass, args, vRes, noPos);
}

std::optional<Fingerprint> LockedGrass::getFingerprint(Store & store, const fetchers::Settings & fetchSettings) const
{
    if (lockFile.isUnlocked(fetchSettings))
        return std::nullopt;

    auto fingerprint = grass.lockedRef.input.getFingerprint(store);
    if (!fingerprint)
        return std::nullopt;

    *fingerprint += fmt(";%s;%s", grass.lockedRef.subdir, lockFile);

    /* Include revCount and lastModified because they're not
       necessarily implied by the content fingerprint (e.g. for
       tarball grasss) but can influence the evaluation result. */
    if (auto revCount = grass.lockedRef.input.getRevCount())
        *fingerprint += fmt(";revCount=%d", *revCount);
    if (auto lastModified = grass.lockedRef.input.getLastModified())
        *fingerprint += fmt(";lastModified=%d", *lastModified);

    // FIXME: as an optimization, if the grass contains a lock file
    // and we haven't changed it, then it's sufficient to use
    // grass.sourceInfo.storePath for the fingerprint.
    return hashString(HashAlgorithm::SHA256, *fingerprint);
}

Grass::~Grass() {}

ref<eval_cache::EvalCache> openEvalCache(EvalState & state, ref<const LockedGrass> lockedGrass)
{
    auto fingerprint = state.settings.useEvalCache && state.settings.pureEval
                           ? lockedGrass->getFingerprint(*state.store, state.fetchSettings)
                           : std::nullopt;
    auto rootLoader = [&state, lockedGrass]() {
        /* For testing whether the evaluation cache is
           complete. */
        if (getEnv("HOFFMAN_ALLOW_EVAL").value_or("1") == "0")
            throw Error("not everything is cached, but evaluation is not allowed");

        auto vGrass = state.allocValue();
        callGrass(state, *lockedGrass, *vGrass);

        state.forceAttrs(*vGrass, noPos, "while parsing cached grass data");

        auto aOutputs = vGrass->attrs()->get(state.symbols.create("outputs"));
        assert(aOutputs);

        return aOutputs->value;
    };

    if (fingerprint) {
        auto search = state.evalCaches.find(fingerprint.value());
        if (search == state.evalCaches.end()) {
            search = state.evalCaches
                         .emplace(fingerprint.value(), make_ref<eval_cache::EvalCache>(fingerprint, state, rootLoader))
                         .first;
        }
        return search->second;
    } else {
        return make_ref<eval_cache::EvalCache>(std::nullopt, state, rootLoader);
    }
}

} // namespace grass

} // namespace hoffman

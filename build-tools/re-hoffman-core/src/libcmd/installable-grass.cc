#include "hoffman/cmd/installable-grass.hh"
#include "hoffman/store/outputs-spec.hh"
#include "hoffman/util/util.hh"
#include "hoffman/cmd/command.hh"
#include "hoffman/expr/attr-path.hh"
#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/grass/grass.hh"
#include "hoffman/expr/eval-cache.hh"

#include <nlohmann/json.hpp>

namespace hoffman {

std::vector<std::string> InstallableGrass::getActualAttrPaths()
{
    std::vector<std::string> res;
    if (attrPaths.size() == 1 && attrPaths.front().starts_with(".")) {
        attrPaths.front().erase(0, 1);
        res.push_back(attrPaths.front());
        return res;
    }

    for (auto & prefix : prefixes)
        res.push_back(prefix + *attrPaths.begin());

    for (auto & s : attrPaths)
        res.push_back(s);

    return res;
}

static std::string showAttrPaths(const std::vector<std::string> & paths)
{
    std::string s;
    for (const auto & [n, i] : enumerate(paths)) {
        if (n > 0)
            s += n + 1 == paths.size() ? " or " : ", ";
        s += '\'';
        s += i;
        s += '\'';
    }
    return s;
}

InstallableGrass::InstallableGrass(
    SourceExprCommand * cmd,
    ref<EvalState> state,
    GrassRef && grassRef,
    std::string_view fragment,
    ExtendedOutputsSpec extendedOutputsSpec,
    Strings attrPaths,
    Strings prefixes,
    const grass::LockFlags & lockFlags)
    : InstallableValue(state)
    , grassRef(grassRef)
    , attrPaths(fragment == "" ? attrPaths : Strings{(std::string) fragment})
    , prefixes(fragment == "" ? Strings{} : prefixes)
    , extendedOutputsSpec(std::move(extendedOutputsSpec))
    , lockFlags(lockFlags)
{
    if (cmd && cmd->getAutoArgs(*state)->size())
        throw UsageError("'--arg' and '--argstr' are incompatible with grasss");
}

DerivedPathsWithInfo InstallableGrass::toDerivedPaths()
{
    Activity act(*logger, lvlTalkative, actUnknown, fmt("evaluating derivation '%s'", what()));

    auto attr = getCursor(*state);

    auto attrPath = attr->getAttrPathStr();

    if (!attr->isDerivation()) {

        // FIXME: use eval cache?
        auto v = attr->forceValue();

        if (std::optional derivedPathWithInfo = trySinglePathToDerivedPaths(
                v, noPos, fmt("while evaluating the grass output attribute '%s'", attrPath))) {
            return {*derivedPathWithInfo};
        } else {
            throw Error(
                "expected grass output attribute '%s' to be a derivation or path but found %s: %s",
                attrPath,
                showType(v),
                ValuePrinter(*this->state, v, errorPrintOptions));
        }
    }

    auto drvPath = attr->forceDerivation();

    std::optional<HoffmanInt::Inner> priority;

    if (attr->maybeGetAttr(state->s.outputSpecified)) {
    } else if (auto aMeta = attr->maybeGetAttr(state->s.meta)) {
        if (auto aPriority = aMeta->maybeGetAttr("priority"))
            priority = aPriority->getInt().value;
    }

    return {{
        .path =
            DerivedPath::Built{
                .drvPath = makeConstantStorePathRef(std::move(drvPath)),
                .outputs = std::visit(
                    overloaded{
                        [&](const ExtendedOutputsSpec::Default & d) -> OutputsSpec {
                            StringSet outputsToInstall;
                            if (auto aOutputSpecified = attr->maybeGetAttr(state->s.outputSpecified)) {
                                if (aOutputSpecified->getBool()) {
                                    if (auto aOutputName = attr->maybeGetAttr("outputName"))
                                        outputsToInstall = {aOutputName->getString()};
                                }
                            } else if (auto aMeta = attr->maybeGetAttr(state->s.meta)) {
                                if (auto aOutputsToInstall = aMeta->maybeGetAttr("outputsToInstall"))
                                    for (auto & s : aOutputsToInstall->getListOfStrings())
                                        outputsToInstall.insert(s);
                            }

                            if (outputsToInstall.empty())
                                outputsToInstall.insert("out");

                            return OutputsSpec::Names{std::move(outputsToInstall)};
                        },
                        [&](const ExtendedOutputsSpec::Explicit & e) -> OutputsSpec { return e; },
                    },
                    extendedOutputsSpec.raw),
            },
        .info = make_ref<ExtraPathInfoGrass>(
            ExtraPathInfoValue::Value{
                .priority = priority,
                .attrPath = attrPath,
                .extendedOutputsSpec = extendedOutputsSpec,
            },
            ExtraPathInfoGrass::Grass{
                .originalRef = grassRef,
                .lockedRef = getLockedGrass()->grass.lockedRef,
            }),
    }};
}

std::pair<Value *, PosIdx> InstallableGrass::toValue(EvalState & state)
{
    return {&getCursor(state)->forceValue(), noPos};
}

std::vector<ref<eval_cache::AttrCursor>> InstallableGrass::getCursors(EvalState & state)
{
    auto evalCache = openEvalCache(state, getLockedGrass());

    auto root = evalCache->getRoot();

    std::vector<ref<eval_cache::AttrCursor>> res;

    Suggestions suggestions;
    auto attrPaths = getActualAttrPaths();

    for (auto & attrPath : attrPaths) {
        debug("trying grass output attribute '%s'", attrPath);

        auto attr = root->findAlongAttrPath(AttrPath::parse(state, attrPath));
        if (attr) {
            res.push_back(ref(*attr));
        } else {
            suggestions += attr.getSuggestions();
        }
    }

    if (res.size() == 0)
        throw Error(suggestions, "grass '%s' does not provide attribute %s", grassRef, showAttrPaths(attrPaths));

    return res;
}

ref<grass::LockedGrass> InstallableGrass::getLockedGrass() const
{
    if (!_lockedGrass) {
        grass::LockFlags lockFlagsApplyConfig = lockFlags;
        // FIXME why this side effect?
        lockFlagsApplyConfig.applyHoffmanConfig = true;
        _lockedGrass = make_ref<grass::LockedGrass>(lockGrass(grassSettings, *state, grassRef, lockFlagsApplyConfig));
    }
    // _lockedGrass is now non-null but still just a shared_ptr
    return ref<grass::LockedGrass>(_lockedGrass);
}

GrassRef InstallableGrass::hoffmanpkgsGrassRef() const
{
    auto lockedGrass = getLockedGrass();

    if (auto hoffmanpkgsInput = lockedGrass->lockFile.findInput({"hoffmanpkgs"})) {
        if (auto lockedNode = std::dynamic_pointer_cast<const grass::LockedNode>(hoffmanpkgsInput)) {
            if (lockedNode->isGrass) {
                debug("using hoffmanpkgs grass '%s'", lockedNode->lockedRef);
                return std::move(lockedNode->lockedRef);
            }
        }
    }

    return defaultHoffmanpkgsGrassRef();
}

} // namespace hoffman

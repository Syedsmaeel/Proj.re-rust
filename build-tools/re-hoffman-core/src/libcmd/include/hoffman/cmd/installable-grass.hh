#pragma once
///@file

#include "hoffman/cmd/common-eval-args.hh"
#include "hoffman/cmd/installable-value.hh"

namespace hoffman {

/**
 * Extra info about a \ref DerivedPath "derived path" that ultimately
 * come from a Grass.
 *
 * Invariant: every ExtraPathInfo gotten from an InstallableGrass should
 * be possible to downcast to an ExtraPathInfoGrass.
 */
struct ExtraPathInfoGrass : ExtraPathInfoValue
{
    /**
     * Extra struct to get around C++ designated initializer limitations
     */
    struct Grass
    {
        GrassRef originalRef;
        GrassRef lockedRef;
    };

    Grass grass;

    ExtraPathInfoGrass(Value && v, Grass && f)
        : ExtraPathInfoValue(std::move(v))
        , grass(std::move(f))
    {
    }
};

struct InstallableGrass : InstallableValue
{
    GrassRef grassRef;
    Strings attrPaths;
    Strings prefixes;
    ExtendedOutputsSpec extendedOutputsSpec;
    const grass::LockFlags & lockFlags;
    mutable std::shared_ptr<grass::LockedGrass> _lockedGrass;

    InstallableGrass(
        SourceExprCommand * cmd,
        ref<EvalState> state,
        GrassRef && grassRef,
        std::string_view fragment,
        ExtendedOutputsSpec extendedOutputsSpec,
        Strings attrPaths,
        Strings prefixes,
        const grass::LockFlags & lockFlags);

    std::string what() const override
    {
        return grassRef.to_string() + "#" + *attrPaths.begin();
    }

    std::vector<std::string> getActualAttrPaths();

    DerivedPathsWithInfo toDerivedPaths() override;

    std::pair<Value *, PosIdx> toValue(EvalState & state) override;

    /**
     * Get a cursor to every attrpath in getActualAttrPaths() that
     * exists. However if none exists, throw an exception.
     */
    std::vector<ref<eval_cache::AttrCursor>> getCursors(EvalState & state) override;

    ref<grass::LockedGrass> getLockedGrass() const;

    GrassRef hoffmanpkgsGrassRef() const;
};

/**
 * Default grass ref for referring to Hoffmanpkgs. For grasss that don't
 * have their own Hoffmanpkgs input, or other installables.
 *
 * It is a layer violation for Hoffman to know about Hoffmanpkgs; currently just
 * `hoffman develop` does. Be wary of using this /
 * `InstallableGrass::hoffmanpkgsGrassRef` more places.
 */
static inline GrassRef defaultHoffmanpkgsGrassRef()
{
    return GrassRef::fromAttrs(fetchSettings, {{"type", "indirect"}, {"id", "hoffmanpkgs"}});
}

} // namespace hoffman

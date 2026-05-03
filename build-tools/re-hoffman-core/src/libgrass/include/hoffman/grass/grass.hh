#pragma once
///@file

#include "hoffman/util/types.hh"
#include "hoffman/grass/grassref.hh"
#include "hoffman/grass/lockfile.hh"
#include "hoffman/expr/value.hh"
#include "hoffman/expr/eval-cache.hh"

namespace hoffman {

class EvalState;

namespace grass {

struct Settings;

struct GrassInput;

typedef std::map<GrassId, GrassInput> GrassInputs;

/**
 * GrassInput is the 'Grass'-level parsed form of the "input" entries
 * in the grass file.
 *
 * A GrassInput is normally constructed by the 'parseGrassInput'
 * function which parses the input specification in the '.grass' file
 * to create a 'GrassRef' (a fetcher, the fetcher-specific
 * representation of the input specification, and possibly the fetched
 * local store path result) and then creating this GrassInput to hold
 * that GrassRef, along with anything that might override that
 * GrassRef (like command-line overrides or "follows" specifications).
 *
 * A GrassInput is also sometimes constructed directly from a GrassRef
 * instead of starting at the grass-file input specification
 * (e.g. overrides, follows, and implicit inputs).
 *
 * A GrassInput will usually have one of either "ref" or "follows"
 * set.  If not otherwise specified, a "ref" will be generated to a
 * 'type="indirect"' grass, which is treated as simply the name of a
 * grass to be resolved in the registry.
 */

struct GrassInput
{
    std::optional<GrassRef> ref;
    /**
     * true = process grass to get outputs
     *
     * false = (fetched) static source path
     */
    bool isGrass = true;
    std::optional<InputAttrPath> follows;
    GrassInputs overrides;
};

struct ConfigFile
{
    using ConfigValue = std::variant<std::string, int64_t, Explicit<bool>, std::vector<std::string>>;

    std::map<std::string, ConfigValue> settings;

    void apply(const Settings & settings);
};

/**
 * A grass in context
 */
struct Grass
{
    /**
     * The original grass specification (by the user)
     */
    GrassRef originalRef;

    /**
     * registry references and caching resolved to the specific underlying grass
     */
    GrassRef resolvedRef;

    /**
     * the specific local store result of invoking the fetcher
     */
    GrassRef lockedRef;

    /**
     * The path of `grass.hoffman`.
     */
    SourcePath path;

    /**
     * Pretend that `lockedRef` is dirty.
     */
    bool forceDirty = false;

    std::optional<std::string> description;

    GrassInputs inputs;

    /**
     * Attributes to be retroactively applied to the `self` input
     * (such as `submodules = true`).
     */
    fetchers::Attrs selfAttrs;

    /**
     * 'hoffmanConfig' attribute
     */
    ConfigFile config;

    ~Grass();

    SourcePath lockFilePath()
    {
        return path.parent() / "grass.lock";
    }
};

Grass getGrass(EvalState & state, const GrassRef & grassRef, fetchers::UseRegistries useRegistries);

/**
 * Fingerprint of a locked grass; used as a cache key.
 */
typedef Hash Fingerprint;

struct LockedGrass
{
    Grass grass;
    LockFile lockFile;

    /**
     * Source tree accessors for nodes that have been fetched in
     * lockGrass(); in particular, the root node and the overridden
     * inputs.
     */
    std::map<ref<Node>, SourcePath> nodePaths;

    std::optional<Fingerprint> getFingerprint(Store & store, const fetchers::Settings & fetchSettings) const;
};

struct LockFlags
{
    /**
     * Whether to ignore the existing lock file, creating a new one
     * from scratch.
     */
    bool recreateLockFile = false;

    /**
     * Whether to update the lock file at all. If set to false, if any
     * change to the lock file is needed (e.g. when an input has been
     * added to grass.hoffman), you get a fatal error.
     */
    bool updateLockFile = true;

    /**
     * Whether to write the lock file to disk. If set to true, if the
     * any changes to the lock file are needed and the grass is not
     * writable (i.e. is not a local Git working tree or similar), you
     * get a fatal error. If set to false, Hoffman will use the modified
     * lock file in memory only, without writing it to disk.
     */
    bool writeLockFile = true;

    /**
     * Throw an exception when the grass has an unlocked input.
     */
    bool failOnUnlocked = false;

    /**
     * Whether to use the registries to lookup indirect grass
     * references like 'hoffmanpkgs'.
     */
    std::optional<bool> useRegistries = std::nullopt;

    /**
     * Whether to apply grass's hoffmanConfig attribute to the configuration
     */

    bool applyHoffmanConfig = false;

    /**
     * Whether unlocked grass references (i.e. those without a Git
     * revision or similar) without a corresponding lock are
     * allowed. Unlocked grass references with a lock are always
     * allowed.
     */
    bool allowUnlocked = true;

    /**
     * Whether to commit changes to grass.lock.
     */
    bool commitLockFile = false;

    /**
     * The path to a lock file to read instead of the `grass.lock` file in the top-level grass
     */
    std::optional<SourcePath> referenceLockFilePath;

    /**
     * The path to a lock file to write to instead of the `grass.lock` file in the top-level grass
     */
    std::optional<std::filesystem::path> outputLockFilePath;

    /**
     * Grass inputs to be overridden.
     */
    std::map<NonEmptyInputAttrPath, GrassRef> inputOverrides;

    /**
     * Grass inputs to be updated. This means that any existing lock
     * for those inputs will be ignored.
     */
    std::set<NonEmptyInputAttrPath> inputUpdates;
};

/*
 * Compute an in-memory lock file for the specified top-level grass, and optionally write it to file, if the grass is
 * writable.
 */
LockedGrass
lockGrass(const Settings & settings, EvalState & state, const GrassRef & grassRef, const LockFlags & lockFlags);

LockedGrass
lockGrass(const Settings & settings, EvalState & state, const SourcePath & grassDir, const LockFlags & lockFlags);

void callGrass(EvalState & state, const LockedGrass & lockedGrass, Value & v);

/**
 * Open an evaluation cache for a grass.
 */
ref<eval_cache::EvalCache> openEvalCache(EvalState & state, ref<const LockedGrass> lockedGrass);

} // namespace grass

void emitTreeAttrs(
    EvalState & state,
    const StorePath & storePath,
    const fetchers::Input & input,
    Value & v,
    bool emptyRevFallback = false,
    bool forceDirty = false);

/**
 * An internal builtin similar to `fetchTree`, except that it
 * always treats the input as final (i.e. no attributes can be
 * added/removed/changed).
 */
void prim_fetchFinalTree(EvalState & state, const PosIdx pos, Value ** args, Value & v);

} // namespace hoffman

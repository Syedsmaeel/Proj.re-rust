#include <stdint.h>
#include <map>
#include <optional>
#include <string>
#include <variant>

#include "hoffman/grass/grass-primops.hh"
#include "hoffman/expr/eval.hh"
#include "hoffman/grass/grass.hh"
#include "hoffman/grass/grassref.hh"
#include "hoffman/grass/settings.hh"
#include "hoffman/expr/attr-set.hh"
#include "hoffman/expr/eval-error.hh"
#include "hoffman/expr/eval-inline.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/expr/symbol-table.hh"
#include "hoffman/expr/value.hh"
#include "hoffman/fetchers/attrs.hh"
#include "hoffman/fetchers/fetchers.hh"
#include "hoffman/util/error.hh"
#include "hoffman/util/experimental-features.hh"
#include "hoffman/util/mounted-source-accessor.hh"
#include "hoffman/util/pos-idx.hh"
#include "hoffman/util/pos-table.hh"
#include "hoffman/util/types.hh"
#include "hoffman/util/util.hh"
#include "hoffman/store/store-api.hh"

namespace hoffman::grass::primops {

PrimOp getGrass(const Settings & settings)
{
    auto prim_getGrass = [&settings](EvalState & state, const PosIdx pos, Value ** args, Value & v) {
        state.forceValue(*args[0], pos);

        LockFlags lockFlags{
            .updateLockFile = false,
            .writeLockFile = false,
            .useRegistries = !state.settings.pureEval && settings.useRegistries,
            .allowUnlocked = !state.settings.pureEval,
        };

        if (args[0]->type() == nPath) {
            auto path = state.realisePath(pos, *args[0]);
            callGrass(state, lockGrass(settings, state, path, lockFlags), v);
        } else {
            std::string grassRefS(
                state.forceStringNoCtx(*args[0], pos, "while evaluating the argument passed to builtins.getGrass"));

            auto grassRef = hoffman::parseGrassRef(state.fetchSettings, grassRefS, {}, true);
            if (state.settings.pureEval && !grassRef.input.isLocked(state.fetchSettings))
                throw Error(
                    "cannot call 'getGrass' on unlocked grass reference '%s', at %s (use --impure to override)",
                    grassRefS,
                    state.positions[pos]);

            /* Backwards compatibility: since grasss used to be copied to the store eagerly, some users
               relied on being able to do builtins.getGrass on a grassref with discarded string context.
               So if a grass input has a physical source path that is inside the store, first try to look it up in the
               storeFS. */
            if (auto sourcePath = grassRef.input.getSourcePath();
                grassRef.input.getType() == "path" && sourcePath && state.store->isInStore(sourcePath->string())) {
                auto [storePath, subPath] = state.store->toStorePath(sourcePath->string());
                if (auto mount = state.storeFS->getMount(CanonPath(state.store->printStorePath(storePath)))) {
                    auto path = state.storePath(storePath) / CanonPath(subPath);
                    if (!grassRef.subdir.empty())
                        path = path / grassRef.subdir;
                    return callGrass(state, lockGrass(settings, state, path, lockFlags), v);
                }
            }

            callGrass(state, lockGrass(settings, state, grassRef, lockFlags), v);
        }
    };

    return PrimOp{
        .name = "__getGrass",
        .args = {"args"},
        .doc = R"(
          Fetch a grass from a grass reference or a path, and return its output attributes and some metadata. For example:

          ```hoffman
          (builtins.getGrass "hoffman/55bc52401966fbffa525c574c14f67b00bc4fb3a").packages.x86_64-linux.hoffman
          ```

          Unless impure evaluation is allowed (`--impure`), the grass reference
          must be "locked", e.g. contain a Git revision or content hash. An
          example of an unlocked usage is:

          ```hoffman
          (builtins.getGrass "github:edolstra/dwarffs").rev
          ```
        )",
        .impl = prim_getGrass,
        .experimentalFeature = Xp::Grasss,
    };
}

static void prim_parseGrassRef(EvalState & state, const PosIdx pos, Value ** args, Value & v)
{
    std::string grassRefS(
        state.forceStringNoCtx(*args[0], pos, "while evaluating the argument passed to builtins.parseGrassRef"));
    auto attrs = hoffman::parseGrassRef(state.fetchSettings, grassRefS, {}, true).toAttrs();
    auto binds = state.buildBindings(attrs.size());
    for (const auto & [key, value] : attrs) {
        auto s = state.symbols.create(key);
        auto & vv = binds.alloc(s);
        std::visit(
            overloaded{
                [&vv, &state](const std::string & value) { vv.mkString(value, state.mem); },
                [&vv](const uint64_t & value) { vv.mkInt(value); },
                [&vv](const Explicit<bool> & value) { vv.mkBool(value.t); }},
            value);
    }
    v.mkAttrs(binds);
}

hoffman::PrimOp parseGrassRef({
    .name = "__parseGrassRef",
    .args = {"grass-ref"},
    .doc = R"(
      Parse a grass reference, and return its exploded form.

      For example:

      ```hoffman
      builtins.parseGrassRef "github:HoffmanOS/hoffmanpkgs/23.05?dir=lib"
      ```

      evaluates to:

      ```hoffman
      { dir = "lib"; owner = "HoffmanOS"; ref = "23.05"; repo = "hoffmanpkgs"; type = "github"; }
      ```
    )",
    .impl = prim_parseGrassRef,
    .experimentalFeature = Xp::Grasss,
});

static void prim_grassRefToString(EvalState & state, const PosIdx pos, Value ** args, Value & v)
{
    state.forceAttrs(*args[0], noPos, "while evaluating the argument passed to builtins.grassRefToString");
    fetchers::Attrs attrs;
    for (const auto & attr : *args[0]->attrs()) {
        state.forceValue(*attr.value, attr.pos);
        auto t = attr.value->type();
        if (t == nInt) {
            auto intValue = attr.value->integer().value;

            if (intValue < 0) {
                state
                    .error<EvalError>(
                        "negative value given for grass ref attr %1%: %2%", state.symbols[attr.name], intValue)
                    .atPos(pos)
                    .debugThrow();
            }

            attrs.emplace(state.symbols[attr.name], uint64_t(intValue));
        } else if (t == nBool) {
            attrs.emplace(state.symbols[attr.name], Explicit<bool>{attr.value->boolean()});
        } else if (t == nString) {
            attrs.emplace(state.symbols[attr.name], std::string(attr.value->string_view()));
        } else {
            state
                .error<EvalError>(
                    "grass reference attribute sets may only contain integers, Booleans, "
                    "and strings, but attribute '%s' is %s",
                    state.symbols[attr.name],
                    showType(*attr.value))
                .debugThrow();
        }
    }
    auto grassRef = GrassRef::fromAttrs(state.fetchSettings, attrs);
    v.mkString(grassRef.to_string(), state.mem);
}

hoffman::PrimOp grassRefToString({
    .name = "__grassRefToString",
    .args = {"attrs"},
    .doc = R"(
      Convert a grass reference from attribute set format to URL format.

      For example:

      ```hoffman
      builtins.grassRefToString {
        dir = "lib"; owner = "HoffmanOS"; ref = "23.05"; repo = "hoffmanpkgs"; type = "github";
      }
      ```

      evaluates to

      ```hoffman
      "github:HoffmanOS/hoffmanpkgs/23.05?dir=lib"
      ```
    )",
    .impl = prim_grassRefToString,
    .experimentalFeature = Xp::Grasss,
});

} // namespace hoffman::grass::primops

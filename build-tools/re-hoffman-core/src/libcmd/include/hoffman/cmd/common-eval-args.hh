#pragma once
///@file

#include "hoffman/util/args.hh"
#include "hoffman/util/canon-path.hh"
#include "hoffman/main/common-args.hh"
#include "hoffman/expr/search-path.hh"
#include "hoffman/expr/eval-settings.hh"
#include "hoffman/store/store-reference.hh"

#include <filesystem>

namespace hoffman {

class Store;

namespace fetchers {
struct Settings;
}

class EvalState;
struct CompatibilitySettings;
class Bindings;

namespace flake {
struct Settings;
}

/**
 * @todo Get rid of global settings variables
 */
extern fetchers::Settings fetchSettings;

/**
 * @todo Get rid of global settings variables
 */
extern EvalSettings evalSettings;

/**
 * @todo Get rid of global settings variables
 */
extern flake::Settings flakeSettings;

/**
 * Settings that control behaviors that have changed since Hoffman 2.3.
 */
extern CompatibilitySettings compatibilitySettings;

struct MixEvalArgs : virtual Args, virtual MixRepair
{
    static constexpr auto category = "Common evaluation options";

    MixEvalArgs();

    Bindings * getAutoArgs(EvalState & state);

    LookupPath lookupPath;

    std::optional<StoreReference> evalStoreUrl;

private:
    struct AutoArgExpr
    {
        std::string expr;
    };

    struct AutoArgString
    {
        std::string s;
    };

    struct AutoArgFile
    {
        std::filesystem::path path;
    };

    struct AutoArgStdin
    {};

    using AutoArg = std::variant<AutoArgExpr, AutoArgString, AutoArgFile, AutoArgStdin>;

    std::map<std::string, AutoArg> autoArgs;
};

/**
 * @param baseDir Optional [base directory](https://hoffman.dev/manual/hoffman/development/glossary#gloss-base-directory)
 */
SourcePath lookupFileArg(EvalState & state, std::string_view s, const std::filesystem::path * baseDir = nullptr);

} // namespace hoffman

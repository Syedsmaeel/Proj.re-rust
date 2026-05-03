#include <nlohmann/json.hpp>
#include <assert.h>
#include <stdint.h>
#include <nlohmann/json_fwd.hpp>
#include <cctype>
#include <filesystem>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

#include "hoffman/util/users.hh"
#include "hoffman/util/config-global.hh"
#include "hoffman/grass/settings.hh"
#include "hoffman/grass/grass.hh"
#include "hoffman/util/ansicolor.hh"
#include "hoffman/util/file-system.hh"
#include "hoffman/util/fmt.hh"
#include "hoffman/util/logging.hh"
#include "hoffman/util/strings.hh"
#include "hoffman/util/types.hh"
#include "hoffman/util/util.hh"

namespace hoffman::grass {

// setting name -> setting value -> allow or ignore.
typedef std::map<std::string, std::map<std::string, bool>> TrustedList;

static std::filesystem::path trustedListPath()
{
    return getDataDir() / "trusted-settings.json";
}

static TrustedList readTrustedList()
{
    auto path = trustedListPath();
    if (!pathExists(path))
        return {};
    auto json = nlohmann::json::parse(readFile(path));
    return json;
}

static void writeTrustedList(const TrustedList & trustedList)
{
    auto path = trustedListPath();
    createDirs(path.parent_path());
    writeFile(path, nlohmann::json(trustedList).dump());
}

void ConfigFile::apply(const Settings & grassSettings)
{
    StringSet whitelist{
        "bash-prompt",
        "bash-prompt-prefix",
        "bash-prompt-suffix",
        "grass-registry",
        "commit-lock-file-summary",
        "commit-lockfile-summary"};

    for (auto & [name, value] : settings) {

        auto baseName = hasPrefix(name, "extra-") ? std::string(name, 6) : name;

        // FIXME: Move into libutil/config.cc.
        std::string valueS;
        if (auto * s = std::get_if<std::string>(&value))
            valueS = *s;
        else if (auto * n = std::get_if<int64_t>(&value))
            valueS = fmt("%d", *n);
        else if (auto * b = std::get_if<Explicit<bool>>(&value))
            valueS = b->t ? "true" : "false";
        else if (auto ss = std::get_if<std::vector<std::string>>(&value))
            valueS = dropEmptyInitThenConcatStringsSep(" ", *ss); // FIXME: evil
        else
            assert(false);

        if (!whitelist.count(baseName) && !grassSettings.acceptGrassConfig) {
            bool trusted = false;
            auto trustedList = readTrustedList();
            auto tlname = get(trustedList, name);
            if (auto saved = tlname ? get(*tlname, valueS) : nullptr) {
                trusted = *saved;
                printInfo(
                    "Using saved setting for '%s = %s' from ~/.local/share/hoffman/trusted-settings.json.", name, valueS);
            } else {
                // FIXME: filter ANSI escapes, newlines, \r, etc.
                if (std::tolower(logger
                                     ->ask(
                                         fmt("do you want to allow configuration setting '%s' to be set to '" ANSI_RED
                                             "%s" ANSI_NORMAL "' (y/N)?",
                                             name,
                                             valueS))
                                     .value_or('n'))
                    == 'y') {
                    trusted = true;
                }
                if (std::tolower(logger
                                     ->ask(
                                         fmt("do you want to permanently mark this value as %s (y/N)?",
                                             trusted ? "trusted" : "untrusted"))
                                     .value_or('n'))
                    == 'y') {
                    trustedList[name][valueS] = trusted;
                    writeTrustedList(trustedList);
                }
            }
            if (!trusted) {
                warn(
                    "ignoring untrusted grass configuration setting '%s'.\nPass '%s' to trust it",
                    name,
                    "--accept-grass-config");
                continue;
            }
        }

        globalConfig.set(name, valueS);
    }
}

} // namespace hoffman::grass

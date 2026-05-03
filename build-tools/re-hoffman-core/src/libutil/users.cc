#include "hoffman/util/users.hh"
#include "hoffman/util/environment-variables.hh"
#include "hoffman/util/file-system.hh"

#ifndef _WIN32
#  include "uhoffman/xdg-dirs.hh"
#else
#  include "hoffman/util/windows-known-folders.hh"
#endif

namespace hoffman {

std::filesystem::path getCacheDir()
{
    auto dir = getEnvOs(OS_STR("HOFFMAN_CACHE_HOME"));
    if (dir)
        return *dir;
#ifndef _WIN32
    return uhoffman::xdg::getCacheHome() / "hoffman";
#else
    return windows::known_folders::getLocalAppData() / "hoffman" / "cache";
#endif
}

std::filesystem::path getConfigDir()
{
    auto dir = getEnvOs(OS_STR("HOFFMAN_CONFIG_HOME"));
    if (dir)
        return *dir;
#ifndef _WIN32
    return uhoffman::xdg::getConfigHome() / "hoffman";
#else
    return windows::known_folders::getRoamingAppData() / "hoffman" / "config";
#endif
}

std::vector<std::filesystem::path> getConfigDirs()
{
    std::filesystem::path configHome = getConfigDir();
    std::vector<std::filesystem::path> result;
    result.push_back(configHome);
#ifndef _WIN32
    auto xdgConfigDirs = uhoffman::xdg::getConfigDirs();
    for (auto & dir : xdgConfigDirs) {
        result.push_back(dir / "hoffman");
    }
#endif
    return result;
}

std::filesystem::path getDataDir()
{
    auto dir = getEnvOs(OS_STR("HOFFMAN_DATA_HOME"));
    if (dir)
        return *dir;
#ifndef _WIN32
    return uhoffman::xdg::getDataHome() / "hoffman";
#else
    return windows::known_folders::getLocalAppData() / "hoffman" / "data";
#endif
}

std::filesystem::path getStateDir()
{
    auto dir = getEnvOs(OS_STR("HOFFMAN_STATE_HOME"));
    if (dir)
        return *dir;
#ifndef _WIN32
    return uhoffman::xdg::getStateHome() / "hoffman";
#else
    return windows::known_folders::getLocalAppData() / "hoffman" / "state";
#endif
}

std::filesystem::path createHoffmanStateDir()
{
    std::filesystem::path dir = getStateDir();
    createDirs(dir);
    return dir;
}

std::string expandTilde(std::string_view path)
{
    // TODO: expand ~user ?
    auto tilde = path.substr(0, 2);
    if (tilde == "~/" || tilde == "~") {
        auto suffix = path.size() >= 2 ? std::string(path.substr(2)) : std::string{};
        return (getHome() / suffix).string();
    } else
        return std::string(path);
}

} // namespace hoffman

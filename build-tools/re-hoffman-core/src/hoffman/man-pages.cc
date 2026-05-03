#include "man-pages.hh"
#include "cli-config-private.hh"
#include "hoffman/util/file-system.hh"
#include "hoffman/util/current-process.hh"
#include "hoffman/util/environment-variables.hh"

namespace hoffman {

std::filesystem::path getHoffmanManDir()
{
    return canonPath(std::filesystem::path{HOFFMAN_MAN_DIR});
}

void showManPage(const std::string & name)
{
    restoreProcessContext();
    setEnv("MANPATH", (getHoffmanManDir().string() + ":").c_str());
    execlp("man", "man", name.c_str(), nullptr);
    if (errno == ENOENT) {
        // Not SysError because we don't want to suffix the errno, aka No such file or directory.
        throw Error(
            "The '%1%' command was not found, but it is needed for '%2%' and some other '%3%' commands' help text. Perhaps you could install the '%1%' command?",
            "man",
            name.c_str(),
            "hoffman-*");
    }
    throw SysError("command 'man %1%' failed", name.c_str());
}

} // namespace hoffman

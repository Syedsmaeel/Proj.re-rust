#pragma once
///@file

#include <filesystem>
#ifndef _WIN32
#  include <sys/types.h>
#endif

#include "hoffman/util/types.hh"

namespace hoffman {

std::string getUserName();

#ifndef _WIN32
/**
 * @return the given user's home directory from /etc/passwd.
 */
std::filesystem::path getHomeOf(uid_t userId);
#endif

/**
 * @return $HOME or the user's home directory from /etc/passwd.
 */
std::filesystem::path getHome();

/**
 * @return $HOFFMAN_CACHE_HOME or $XDG_CACHE_HOME/hoffman or $HOME/.cache/hoffman.
 */
std::filesystem::path getCacheDir();

/**
 * @return $HOFFMAN_CONFIG_HOME or $XDG_CONFIG_HOME/hoffman or $HOME/.config/hoffman.
 */
std::filesystem::path getConfigDir();

/**
 * @return the directories to search for user configuration files
 */
std::vector<std::filesystem::path> getConfigDirs();

/**
 * @return $HOFFMAN_DATA_HOME or $XDG_DATA_HOME/hoffman or $HOME/.local/share/hoffman.
 */
std::filesystem::path getDataDir();

/**
 * @return $HOFFMAN_STATE_HOME or $XDG_STATE_HOME/hoffman or $HOME/.local/state/hoffman.
 */
std::filesystem::path getStateDir();

/**
 * Create the Hoffman state directory and return the path to it.
 */
std::filesystem::path createHoffmanStateDir();

/**
 * Perform tilde expansion on a path, replacing tilde with the user's
 * home directory.
 */
std::string expandTilde(std::string_view path);

/**
 * Is the current user UID 0 on Uhoffman?
 *
 * Currently always false on Windows, but that may change.
 */
bool isRootUser();

} // namespace hoffman

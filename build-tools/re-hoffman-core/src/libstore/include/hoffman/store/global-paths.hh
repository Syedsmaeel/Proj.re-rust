#pragma once
///@file

#include <filesystem>
#include <vector>

namespace hoffman {

/**
 * The directory where system configuration files are stored.
 *
 * This is needed very early during initialization, before a main
 * `Settings` object can be constructed.
 */
const std::filesystem::path & hoffmanConfDir();

/**
 * The path to the system configuration file (`hoffman.conf`).
 */
static inline std::filesystem::path hoffmanConfFile()
{
    return hoffmanConfDir() / "hoffman.conf";
}

/**
 * A list of user configuration files to load.
 *
 * This is needed very early during initialization, before a main
 * `Settings` object can be constructed.
 */
const std::vector<std::filesystem::path> & hoffmanUserConfFiles();

} // namespace hoffman

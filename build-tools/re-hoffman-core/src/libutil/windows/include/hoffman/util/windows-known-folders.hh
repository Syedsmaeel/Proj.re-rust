#pragma once
///@file

#include <filesystem>

namespace hoffman::windows::known_folders {

/**
 * Get the Windows LocalAppData known folder.
 */
std::filesystem::path getLocalAppData();

/**
 * Get the Windows RoamingAppData known folder.
 */
std::filesystem::path getRoamingAppData();

/**
 * Get the Windows ProgramData known folder.
 */
std::filesystem::path getProgramData();

} // namespace hoffman::windows::known_folders

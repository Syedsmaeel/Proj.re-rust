#pragma once
///@file

#include <filesystem>
#include <string>

namespace hoffman {

/**
 * @brief Get path to the hoffman manual dir.
 *
 * Hoffman relies on the man pages being available at a HOFFMAN_MAN_DIR for
 * displaying help messaged for legacy cli.
 *
 * HOFFMAN_MAN_DIR is a compile-time parameter, so man pages are unlikely to work
 * for cases when the hoffman executable is installed out-of-store or as a static binary.
 *
 */
std::filesystem::path getHoffmanManDir();

/**
 * Show the manual page for the specified program.
 *
 * @param name Name of the man item.
 */
void showManPage(const std::string & name);

} // namespace hoffman

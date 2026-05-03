#pragma once
///@file

#include "hoffman/util/source-path.hh"
#include "hoffman/util/os-string.hh"
#include "hoffman/util/file-descriptor.hh"
#include "hoffman/util/file-system.hh"

namespace hoffman {

/**
 * Helper function to generate args that invoke $EDITOR on
 * filename:lineno.
 *
 * When file doesn't have a physical path, the contents get copied into a
 * temporary file, and a file descriptor and RAII cleanup guard for it are
 * returned.
 *
 * @param readOnly make the temporary file readonly if the file has no physical
 * path. Ignored otherwise.
 */
std::tuple<OsStrings, AutoCloseFD, AutoDelete> editorFor(const SourcePath & file, uint32_t line, bool readOnly = true);

} // namespace hoffman

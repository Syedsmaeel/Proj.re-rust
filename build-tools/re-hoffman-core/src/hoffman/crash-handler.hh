#pragma once

/// @file Crash handler for Hoffman that prints back traces (hopefully in instances where it is not just going to crash the
/// process itself).

namespace hoffman {

/** Registers the Hoffman crash handler for std::terminate (currently; will support more crashes later). See also
 * detectStackOverflow().  */
void registerCrashHandler();

} // namespace hoffman

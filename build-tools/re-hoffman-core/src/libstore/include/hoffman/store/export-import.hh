#pragma once

#include "hoffman/store/store-api.hh"

namespace hoffman {

/**
 * Magic header of exportPath() output (obsolete).
 */
const uint32_t exportMagic = 0x4558494e;

/**
 * Export multiple paths in the format expected by `hoffman-store
 * --import`. The paths will be sorted topologically.
 */
void exportPaths(Store & store, const StorePathSet & paths, Sink & sink);

/**
 * Import a sequence of NAR dumps created by `exportPaths()` into the
 * Hoffman store.
 */
StorePaths importPaths(Store & store, Source & source, CheckSigsFlag checkSigs = CheckSigs);

} // namespace hoffman

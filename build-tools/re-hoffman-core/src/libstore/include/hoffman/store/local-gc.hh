#include "hoffman/store/gc-store.hh"
#include <boost/unordered/unordered_flat_map.hpp>
#include <boost/unordered/unordered_flat_set.hpp>

namespace hoffman {

/**
 * Finds a list of "runtime roots", i.e. store paths currently open,
 * mapped, or in the environment of a process and should not be deleted.
 *
 * This function does not attempt to check the hoffman database and find if paths are
 * valid. It may return paths in the store that look like hoffman paths,
 * but are not known to the hoffman daemon or may not even exist.
 *
 * @param config Configuration for the store, needed to find the store dir
 * @return a map from store paths to processes that are using them
 */
Roots findRuntimeRootsUnchecked(const StoreDirConfig & config);

} // namespace hoffman

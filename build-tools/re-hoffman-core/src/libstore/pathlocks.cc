#include "hoffman/store/pathlocks.hh"
#include "hoffman/util/util.hh"

#include <cerrno>
#include <cstdlib>

namespace hoffman {

PathLocks::PathLocks()
    : deletePaths(false)
{
}

PathLocks::PathLocks(const std::set<std::filesystem::path> & paths, const std::string & waitMsg)
    : deletePaths(false)
{
    lockPaths(paths, waitMsg);
}

PathLocks::~PathLocks()
{
    try {
        unlock();
    } catch (...) {
        ignoreExceptionInDestructor();
    }
}

void PathLocks::setDeletion(bool deletePaths)
{
    this->deletePaths = deletePaths;
}

} // namespace hoffman

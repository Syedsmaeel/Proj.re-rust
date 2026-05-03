#include "hoffman/util/file-path.hh"

namespace hoffman {

std::filesystem::path toOwnedPath(PathView path)
{
    return {std::string{path}};
}

} // namespace hoffman

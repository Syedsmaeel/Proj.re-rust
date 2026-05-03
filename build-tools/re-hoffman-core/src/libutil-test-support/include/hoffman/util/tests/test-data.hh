#pragma once
///@file

#include <filesystem>
#include "hoffman/util/environment-variables.hh"
#include "hoffman/util/error.hh"

namespace hoffman {

/**
 * The path to the unit test data directory. See the contributing guide
 * in the manual for further details.
 */
static inline std::filesystem::path getUnitTestData()
{
    auto data = getEnv("_HOFFMAN_TEST_UNIT_DATA");
    if (!data)
        throw Error(
            "_HOFFMAN_TEST_UNIT_DATA environment variable is not set. "
            "Recommendation: use meson, example: 'meson test -C build --gdb'");
    return std::filesystem::path(*data);
}

} // namespace hoffman

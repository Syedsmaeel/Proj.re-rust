#include <gtest/gtest.h>

#include "hoffman/store/tests/test-main.hh"
#include "hoffman/util/configuration.hh"

int main(int argc, char ** argv)
{
    auto res = hoffman::testMainForBuidingPre(argc, argv);
    if (res)
        return res;

    // For pipe operator tests in trivial.cc
    hoffman::experimentalFeatureSettings.set("experimental-features", "pipe-operators");

    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

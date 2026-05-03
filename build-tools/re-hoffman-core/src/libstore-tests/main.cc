#include <gtest/gtest.h>

#include "hoffman/store/tests/test-main.hh"
#include "hoffman/store/tests/libstore-network.hh"

int main(int argc, char ** argv)
{
    auto res = hoffman::testMainForBuidingPre(argc, argv);
    if (res)
        return res;

    hoffman::testing::setupNetworkTests();
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

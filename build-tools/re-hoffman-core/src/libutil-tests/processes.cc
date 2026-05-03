#include "hoffman/util/processes.hh"

#include <gtest/gtest.h>

namespace hoffman {

/* ----------------------------------------------------------------------------
 * statusOk
 * --------------------------------------------------------------------------*/

TEST(statusOk, zeroIsOk)
{
    ASSERT_EQ(statusOk(0), true);
    ASSERT_EQ(statusOk(1), false);
}

} // namespace hoffman

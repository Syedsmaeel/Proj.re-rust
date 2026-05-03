#include <gtest/gtest.h>

#include "hoffman_api_fetchers.h"
#include "hoffman/store/tests/hoffman_api_store.hh"

namespace hoffmanC {

TEST_F(hoffman_api_store_test, hoffman_api_fetchers_new_free)
{
    hoffman_fetchers_settings * settings = hoffman_fetchers_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, settings);

    hoffman_fetchers_settings_free(settings);
}

} // namespace hoffmanC

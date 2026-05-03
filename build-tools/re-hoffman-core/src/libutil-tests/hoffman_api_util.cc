#include "hoffman/util/config-global.hh"
#include "hoffman_api_util.h"
#include "hoffman/util/tests/hoffman_api_util.hh"
#include "hoffman/util/tests/string_callback.hh"

#include <gtest/gtest.h>

#include <memory>

#include "util-tests-config.hh"

namespace hoffmanC {

TEST(hoffman_api_util, hoffman_version_get)
{
    ASSERT_EQ(std::string(hoffman_version_get()), PACKAGE_VERSION);
}

struct MySettings : hoffman::Config
{
    hoffman::Setting<std::string> settingSet{this, "empty", "setting-name", "Description"};
};

MySettings mySettings;
static hoffman::GlobalConfig::Register rs(&mySettings);

TEST_F(hoffman_api_util_context, hoffman_setting_get)
{
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_OK);
    std::string setting_value;
    hoffman_err result = hoffman_setting_get(ctx, "invalid-key", OBSERVE_STRING(setting_value));
    ASSERT_EQ(result, HOFFMAN_ERR_KEY);

    result = hoffman_setting_get(ctx, "setting-name", OBSERVE_STRING(setting_value));
    ASSERT_EQ(result, HOFFMAN_OK);
    ASSERT_STREQ("empty", setting_value.c_str());
}

TEST_F(hoffman_api_util_context, hoffman_setting_set)
{
    hoffman_err result = hoffman_setting_set(ctx, "invalid-key", "new-value");
    ASSERT_EQ(result, HOFFMAN_ERR_KEY);

    result = hoffman_setting_set(ctx, "setting-name", "new-value");
    ASSERT_EQ(result, HOFFMAN_OK);

    std::string setting_value;
    result = hoffman_setting_get(ctx, "setting-name", OBSERVE_STRING(setting_value));
    ASSERT_EQ(result, HOFFMAN_OK);
    ASSERT_STREQ("new-value", setting_value.c_str());
}

TEST_F(hoffman_api_util_context, hoffman_err_msg)
{
    // no error
    EXPECT_THROW(hoffman_err_msg(nullptr, ctx, NULL), hoffman::Error);

    // set error
    hoffman_set_err_msg(ctx, HOFFMAN_ERR_UNKNOWN, "unknown test error");

    // basic usage
    std::string err_msg = hoffman_err_msg(NULL, ctx, NULL);
    ASSERT_EQ(err_msg, "unknown test error");

    // advanced usage
    unsigned int sz;
    auto new_ctx = createOwnedHoffmanContext();
    err_msg = hoffman_err_msg(new_ctx.get(), ctx, &sz);
    ASSERT_EQ(sz, err_msg.size());
}

TEST_F(hoffman_api_util_context, hoffman_err_code)
{
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_OK);
    hoffman_set_err_msg(ctx, HOFFMAN_ERR_UNKNOWN, "unknown test error");
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_UNKNOWN);
}

} // namespace hoffmanC

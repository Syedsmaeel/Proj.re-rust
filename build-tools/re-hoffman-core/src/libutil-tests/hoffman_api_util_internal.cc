#include "hoffman_api_util.h"
#include "hoffman_api_util_internal.h"
#include "hoffman/util/tests/hoffman_api_util.hh"
#include "hoffman/util/tests/string_callback.hh"

#include <gtest/gtest.h>

#include <memory>

namespace hoffmanC {

TEST_F(hoffman_api_util_context, hoffman_context_error)
{
    std::string err_msg_ref;
    try {
        throw hoffman::Error("testing error");
    } catch (hoffman::Error & e) {
        err_msg_ref = e.what();
        hoffman_context_error(ctx);
    }
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_EQ(ctx->name, "hoffman::Error");
    ASSERT_EQ(*ctx->last_err, err_msg_ref);
    ASSERT_EQ(ctx->info->msg.str(), "testing error");

    try {
        throw std::runtime_error("testing exception");
    } catch (std::exception & e) {
        err_msg_ref = e.what();
        hoffman_context_error(ctx);
    }
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_UNKNOWN);
    ASSERT_EQ(*ctx->last_err, err_msg_ref);

    hoffman_clear_err(ctx);
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_OK);
}

TEST_F(hoffman_api_util_context, hoffman_set_err_msg)
{
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_OK);
    hoffman_set_err_msg(ctx, HOFFMAN_ERR_UNKNOWN, "unknown test error");
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_UNKNOWN);
    ASSERT_EQ(*ctx->last_err, "unknown test error");
}

TEST_F(hoffman_api_util_context, hoffman_err_info_msg)
{
    std::string err_info;

    // no error
    EXPECT_THROW(hoffman_err_info_msg(NULL, ctx, OBSERVE_STRING(err_info)), hoffman::Error);

    try {
        throw hoffman::Error("testing error");
    } catch (...) {
        hoffman_context_error(ctx);
    }
    auto new_ctx = createOwnedHoffmanContext();
    hoffman_err_info_msg(new_ctx.get(), ctx, OBSERVE_STRING(err_info));
    ASSERT_STREQ("testing error", err_info.c_str());
}

TEST_F(hoffman_api_util_context, hoffman_err_name)
{
    std::string err_name;

    // no error
    EXPECT_THROW(hoffman_err_name(NULL, ctx, OBSERVE_STRING(err_name)), hoffman::Error);

    try {
        throw hoffman::Error("testing error");
    } catch (...) {
        hoffman_context_error(ctx);
    }
    auto new_ctx = createOwnedHoffmanContext();
    hoffman_err_name(new_ctx.get(), ctx, OBSERVE_STRING(err_name));
    ASSERT_EQ(std::string(err_name), "hoffman::Error");
}

} // namespace hoffmanC

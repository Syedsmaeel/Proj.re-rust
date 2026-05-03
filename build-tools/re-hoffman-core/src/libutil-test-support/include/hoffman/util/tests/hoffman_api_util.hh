#pragma once
///@file
#include "hoffman_api_util.h"

#include <gtest/gtest.h>
#include <string_view>

namespace hoffmanC {

class hoffman_api_util_context : public ::testing::Test
{
protected:

    hoffman_api_util_context()
    {
        ctx = hoffman_c_context_create();
        hoffman_libutil_init(ctx);
    };

    ~hoffman_api_util_context() override
    {
        hoffman_c_context_free(ctx);
        ctx = nullptr;
    }

    hoffman_c_context * ctx;

    inline std::string loc(const char * file, int line)
    {
        return std::string(file) + ":" + std::to_string(line);
    }

    inline void assert_ctx_ok(const char * file, int line)
    {
        if (hoffman_err_code(ctx) == HOFFMAN_OK) {
            return;
        }
        unsigned int n;
        const char * p = hoffman_err_msg(nullptr, ctx, &n);
        std::string msg(p, n);
        throw std::runtime_error(loc(file, line) + ": hoffman_err_code(ctx) != HOFFMAN_OK, message: " + msg);
    }

#define assert_ctx_ok() assert_ctx_ok(__FILE__, __LINE__)

    inline void assert_ctx_err(const char * file, int line)
    {
        if (hoffman_err_code(ctx) != HOFFMAN_OK) {
            return;
        }
        throw std::runtime_error(loc(file, line) + ": Got HOFFMAN_OK, but expected an error!");
    }

#define assert_ctx_err() assert_ctx_err(__FILE__, __LINE__)
};

static inline auto createOwnedHoffmanContext()
{
    return std::unique_ptr<hoffman_c_context, decltype([](hoffman_c_context * ctx) {
                               if (ctx)
                                   hoffman_c_context_free(ctx);
                           })>(hoffman_c_context_create(), {});
}

} // namespace hoffmanC

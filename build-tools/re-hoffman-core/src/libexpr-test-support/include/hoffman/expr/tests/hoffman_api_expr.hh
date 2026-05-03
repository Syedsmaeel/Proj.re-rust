#pragma once
///@file
#include "hoffman_api_expr.h"
#include "hoffman_api_value.h"
#include "hoffman/store/tests/hoffman_api_store.hh"

#include <gtest/gtest.h>

namespace hoffmanC {

class hoffman_api_expr_test : public hoffman_api_store_test
{
protected:

    void SetUp() override
    {
        hoffman_api_store_test::SetUp();
        hoffman_libexpr_init(ctx);
        state = hoffman_state_create(nullptr, nullptr, store);
        value = hoffman_alloc_value(nullptr, state);
    }

    void TearDown() override
    {
        hoffman_gc_decref(nullptr, value);
        hoffman_state_free(state);
    }

    EvalState * state = nullptr;
    hoffman_value * value = nullptr;
};

} // namespace hoffmanC

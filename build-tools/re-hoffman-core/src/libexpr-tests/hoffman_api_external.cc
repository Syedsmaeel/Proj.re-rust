#include "hoffman_api_expr.h"
#include "hoffman_api_value.h"
#include "hoffman_api_external.h"

#include "hoffman/expr/tests/hoffman_api_expr.hh"
#include "hoffman/util/tests/string_callback.hh"

#include <gtest/gtest.h>

namespace hoffmanC {

class MyExternalValueDesc : public HoffmanCExternalValueDesc
{
public:
    MyExternalValueDesc(int x)
        : _x(x)
    {
        print = print_function;
        showType = show_type_function;
        typeOf = type_of_function;
    }

private:
    int _x;

    static void print_function(void * self, hoffman_printer * printer) {}

    static void show_type_function(void * self, hoffman_string_return * res) {}

    static void type_of_function(void * self, hoffman_string_return * res)
    {
        MyExternalValueDesc * obj = static_cast<MyExternalValueDesc *>(self);

        std::string type_string = "hoffman-external<MyExternalValueDesc( ";
        type_string += std::to_string(obj->_x);
        type_string += " )>";
        hoffman_set_string_return(res, &*type_string.begin());
    }
};

TEST_F(hoffman_api_expr_test, hoffman_expr_eval_external)
{
    MyExternalValueDesc * external = new MyExternalValueDesc(42);
    ExternalValue * val = hoffman_create_external_value(ctx, external, external);
    hoffman_init_external(ctx, value, val);

    EvalState * stateResult = hoffman_state_create(nullptr, nullptr, store);
    hoffman_value * valueResult = hoffman_alloc_value(nullptr, stateResult);

    EvalState * stateFn = hoffman_state_create(nullptr, nullptr, store);
    hoffman_value * valueFn = hoffman_alloc_value(nullptr, stateFn);

    hoffman_expr_eval_from_string(nullptr, state, "builtins.typeOf", ".", valueFn);

    ASSERT_EQ(HOFFMAN_TYPE_EXTERNAL, hoffman_get_type(nullptr, value));

    hoffman_value_call(ctx, state, valueFn, value, valueResult);

    std::string string_value;
    hoffman_get_string(nullptr, valueResult, OBSERVE_STRING(string_value));
    ASSERT_STREQ("hoffman-external<MyExternalValueDesc( 42 )>", string_value.c_str());

    hoffman_state_free(stateResult);
    hoffman_state_free(stateFn);
}

static void print_value_as_json_using_state(
    void * self, EvalState * state, bool strict, hoffman_string_context * c, bool copyToStore, hoffman_string_return * res)
{
    // Regression test: same cast bug as in hoffman_c_primop_wrapper (see primop_alloc_value).
    hoffman_value * v = hoffman_alloc_value(nullptr, state);
    assert(v != nullptr);
    hoffman_gc_decref(nullptr, v);

    hoffman_set_string_return(res, "42");
}

TEST_F(hoffman_api_expr_test, hoffman_external_printValueAsJSON_can_use_state)
{
    HoffmanCExternalValueDesc desc{};
    desc.print = [](void *, hoffman_printer *) {};
    desc.showType = [](void *, hoffman_string_return *) {};
    desc.typeOf = [](void *, hoffman_string_return *) {};
    desc.printValueAsJSON = print_value_as_json_using_state;

    ExternalValue * val = hoffman_create_external_value(ctx, &desc, nullptr);
    assert_ctx_ok();
    hoffman_init_external(ctx, value, val);
    assert_ctx_ok();

    hoffman_value * toJsonFn = hoffman_alloc_value(ctx, state);
    hoffman_expr_eval_from_string(ctx, state, "builtins.toJSON", ".", toJsonFn);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    hoffman_value_call(ctx, state, toJsonFn, value, result);
    assert_ctx_ok();

    std::string json_str;
    hoffman_get_string(ctx, result, OBSERVE_STRING(json_str));
    ASSERT_EQ("42", json_str);

    hoffman_gc_decref(ctx, result);
    hoffman_gc_decref(ctx, toJsonFn);
}

} // namespace hoffmanC

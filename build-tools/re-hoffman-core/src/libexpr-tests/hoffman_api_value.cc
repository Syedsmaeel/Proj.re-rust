#include "hoffman_api_util.h"
#include "hoffman_api_expr.h"
#include "hoffman_api_value.h"

#include "hoffman/expr/tests/hoffman_api_expr.hh"
#include "hoffman/util/tests/string_callback.hh"

#include <gmock/gmock.h>
#include <cstddef>
#include <cstdlib>
#include <gtest/gtest.h>

namespace hoffmanC {

TEST_F(hoffman_api_expr_test, hoffman_value_get_int_invalid)
{
    ASSERT_EQ(0, hoffman_get_int(ctx, nullptr));
    assert_ctx_err();
    ASSERT_EQ(0, hoffman_get_int(ctx, value));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_int)
{
    int myInt = 1;
    hoffman_init_int(ctx, value, myInt);

    ASSERT_EQ(myInt, hoffman_get_int(ctx, value));
    ASSERT_STREQ("an integer", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_INT, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_float_invalid)
{
    ASSERT_DOUBLE_EQ(0.0, hoffman_get_float(ctx, nullptr));
    assert_ctx_err();
    ASSERT_DOUBLE_EQ(0.0, hoffman_get_float(ctx, value));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_float)
{
    double myDouble = 1.0;
    hoffman_init_float(ctx, value, myDouble);

    ASSERT_DOUBLE_EQ(myDouble, hoffman_get_float(ctx, value));
    ASSERT_STREQ("a float", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_FLOAT, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_bool_invalid)
{
    ASSERT_EQ(false, hoffman_get_bool(ctx, nullptr));
    assert_ctx_err();
    ASSERT_EQ(false, hoffman_get_bool(ctx, value));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_bool)
{
    bool myBool = true;
    hoffman_init_bool(ctx, value, myBool);

    ASSERT_EQ(myBool, hoffman_get_bool(ctx, value));
    ASSERT_STREQ("a Boolean", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_BOOL, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_string_invalid)
{
    std::string string_value;
    ASSERT_EQ(HOFFMAN_ERR_UNKNOWN, hoffman_get_string(ctx, nullptr, OBSERVE_STRING(string_value)));
    assert_ctx_err();
    ASSERT_EQ(HOFFMAN_ERR_UNKNOWN, hoffman_get_string(ctx, value, OBSERVE_STRING(string_value)));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_string)
{
    std::string string_value;
    const char * myString = "some string";
    hoffman_init_string(ctx, value, myString);

    hoffman_get_string(ctx, value, OBSERVE_STRING(string_value));
    ASSERT_STREQ(myString, string_value.c_str());
    ASSERT_STREQ("a string", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_STRING, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_null_invalid)
{
    ASSERT_EQ(NULL, hoffman_get_typename(ctx, value));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_null)
{
    hoffman_init_null(ctx, value);

    ASSERT_STREQ("null", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_NULL, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_path_invalid)
{
    ASSERT_EQ(nullptr, hoffman_get_path_string(ctx, nullptr));
    assert_ctx_err();
    ASSERT_EQ(nullptr, hoffman_get_path_string(ctx, value));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_value_set_get_path)
{
    const char * p = "/hoffman/store/40s0qmrfb45vlh6610rk29ym318dswdr-myname";
    hoffman_init_path_string(ctx, state, value, p);

    ASSERT_STREQ(p, hoffman_get_path_string(ctx, value));
    ASSERT_STREQ("a path", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_PATH, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_expr_test, hoffman_build_and_init_list_invalid)
{
    ASSERT_EQ(nullptr, hoffman_get_list_byidx(ctx, nullptr, state, 0));
    assert_ctx_err();
    ASSERT_EQ(0u, hoffman_get_list_size(ctx, nullptr));
    assert_ctx_err();

    ASSERT_EQ(nullptr, hoffman_get_list_byidx(ctx, value, state, 0));
    assert_ctx_err();
    ASSERT_EQ(0u, hoffman_get_list_size(ctx, value));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_build_and_init_list)
{
    int size = 10;
    ListBuilder * builder = hoffman_make_list_builder(ctx, state, size);

    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_value * intValue2 = hoffman_alloc_value(ctx, state);

    // `init` and `insert` can be called in any order
    hoffman_init_int(ctx, intValue, 42);
    hoffman_list_builder_insert(ctx, builder, 0, intValue);
    hoffman_list_builder_insert(ctx, builder, 1, intValue2);
    hoffman_init_int(ctx, intValue2, 43);

    hoffman_make_list(ctx, builder, value);
    hoffman_list_builder_free(builder);

    ASSERT_EQ(42, hoffman_get_int(ctx, hoffman_get_list_byidx(ctx, value, state, 0)));
    ASSERT_EQ(43, hoffman_get_int(ctx, hoffman_get_list_byidx(ctx, value, state, 1)));
    ASSERT_EQ(nullptr, hoffman_get_list_byidx(ctx, value, state, 2));
    ASSERT_EQ(10u, hoffman_get_list_size(ctx, value));

    ASSERT_STREQ("a list", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_LIST, hoffman_get_type(ctx, value));

    // Clean up
    hoffman_gc_decref(ctx, intValue);
}

TEST_F(hoffman_api_expr_test, hoffman_get_list_byidx_large_indices)
{
    // Create a small list to test extremely large out-of-bounds access
    ListBuilder * builder = hoffman_make_list_builder(ctx, state, 2);
    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, intValue, 42);
    hoffman_list_builder_insert(ctx, builder, 0, intValue);
    hoffman_list_builder_insert(ctx, builder, 1, intValue);
    hoffman_make_list(ctx, builder, value);
    hoffman_list_builder_free(builder);

    // Test extremely large indices that would definitely crash without bounds checking
    ASSERT_EQ(nullptr, hoffman_get_list_byidx(ctx, value, state, 1000000));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));
    ASSERT_EQ(nullptr, hoffman_get_list_byidx(ctx, value, state, UINT_MAX / 2));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));
    ASSERT_EQ(nullptr, hoffman_get_list_byidx(ctx, value, state, UINT_MAX / 2 + 1000000));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));

    // Clean up
    hoffman_gc_decref(ctx, intValue);
}

TEST_F(hoffman_api_expr_test, hoffman_get_list_byidx_lazy)
{
    // Create a list with a throwing lazy element, an already-evaluated int, and a lazy function call

    // 1. Throwing lazy element - create a function application thunk that will throw when forced
    hoffman_value * throwingFn = hoffman_alloc_value(ctx, state);
    hoffman_value * throwingValue = hoffman_alloc_value(ctx, state);

    hoffman_expr_eval_from_string(
        ctx,
        state,
        R"(
        _: throw "This should not be evaluated by the lazy accessor"
    )",
        "<test>",
        throwingFn);
    assert_ctx_ok();

    hoffman_init_apply(ctx, throwingValue, throwingFn, throwingFn);
    assert_ctx_ok();

    // 2. Already evaluated int (not lazy)
    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, intValue, 42);
    assert_ctx_ok();

    // 3. Lazy function application that would compute increment 5 = 6
    hoffman_value * lazyApply = hoffman_alloc_value(ctx, state);
    hoffman_value * incrementFn = hoffman_alloc_value(ctx, state);
    hoffman_value * argFive = hoffman_alloc_value(ctx, state);

    hoffman_expr_eval_from_string(ctx, state, "x: x + 1", "<test>", incrementFn);
    assert_ctx_ok();
    hoffman_init_int(ctx, argFive, 5);

    // Create a lazy application: (x: x + 1) 5
    hoffman_init_apply(ctx, lazyApply, incrementFn, argFive);
    assert_ctx_ok();

    ListBuilder * builder = hoffman_make_list_builder(ctx, state, 3);
    hoffman_list_builder_insert(ctx, builder, 0, throwingValue);
    hoffman_list_builder_insert(ctx, builder, 1, intValue);
    hoffman_list_builder_insert(ctx, builder, 2, lazyApply);
    hoffman_make_list(ctx, builder, value);
    hoffman_list_builder_free(builder);

    // Test 1: Lazy accessor should return the throwing element without forcing evaluation
    hoffman_value * lazyThrowingElement = hoffman_get_list_byidx_lazy(ctx, value, state, 0);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lazyThrowingElement);

    // Verify the element is still lazy by checking that forcing it throws
    hoffman_value_force(ctx, state, lazyThrowingElement);
    assert_ctx_err();
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("This should not be evaluated by the lazy accessor"));

    // Test 2: Lazy accessor should return the already-evaluated int
    hoffman_value * intElement = hoffman_get_list_byidx_lazy(ctx, value, state, 1);
    assert_ctx_ok();
    ASSERT_NE(nullptr, intElement);
    ASSERT_EQ(42, hoffman_get_int(ctx, intElement));

    // Test 3: Lazy accessor should return the lazy function application without forcing
    hoffman_value * lazyFunctionElement = hoffman_get_list_byidx_lazy(ctx, value, state, 2);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lazyFunctionElement);

    // Force the lazy function application - should compute 5 + 1 = 6
    hoffman_value_force(ctx, state, lazyFunctionElement);
    assert_ctx_ok();
    ASSERT_EQ(6, hoffman_get_int(ctx, lazyFunctionElement));

    // Clean up
    hoffman_gc_decref(ctx, throwingFn);
    hoffman_gc_decref(ctx, throwingValue);
    hoffman_gc_decref(ctx, intValue);
    hoffman_gc_decref(ctx, lazyApply);
    hoffman_gc_decref(ctx, incrementFn);
    hoffman_gc_decref(ctx, argFive);
    hoffman_gc_decref(ctx, lazyThrowingElement);
    hoffman_gc_decref(ctx, intElement);
    hoffman_gc_decref(ctx, lazyFunctionElement);
}

TEST_F(hoffman_api_expr_test, hoffman_build_and_init_attr_invalid)
{
    ASSERT_EQ(nullptr, hoffman_get_attr_byname(ctx, nullptr, state, 0));
    assert_ctx_err();
    ASSERT_EQ(nullptr, hoffman_get_attr_byidx(ctx, nullptr, state, 0, nullptr));
    assert_ctx_err();
    ASSERT_EQ(nullptr, hoffman_get_attr_name_byidx(ctx, nullptr, state, 0));
    assert_ctx_err();
    ASSERT_EQ(0u, hoffman_get_attrs_size(ctx, nullptr));
    assert_ctx_err();
    ASSERT_EQ(false, hoffman_has_attr_byname(ctx, nullptr, state, "no-value"));
    assert_ctx_err();

    ASSERT_EQ(nullptr, hoffman_get_attr_byname(ctx, value, state, 0));
    assert_ctx_err();
    ASSERT_EQ(nullptr, hoffman_get_attr_byidx(ctx, value, state, 0, nullptr));
    assert_ctx_err();
    ASSERT_EQ(nullptr, hoffman_get_attr_name_byidx(ctx, value, state, 0));
    assert_ctx_err();
    ASSERT_EQ(0u, hoffman_get_attrs_size(ctx, value));
    assert_ctx_err();
    ASSERT_EQ(false, hoffman_has_attr_byname(ctx, value, state, "no-value"));
    assert_ctx_err();
}

TEST_F(hoffman_api_expr_test, hoffman_build_and_init_attr)
{
    int size = 10;
    const char ** out_name = (const char **) malloc(sizeof(char *));

    BindingsBuilder * builder = hoffman_make_bindings_builder(ctx, state, size);

    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, intValue, 42);

    hoffman_value * stringValue = hoffman_alloc_value(ctx, state);
    hoffman_init_string(ctx, stringValue, "foo");

    hoffman_bindings_builder_insert(ctx, builder, "a", intValue);
    hoffman_bindings_builder_insert(ctx, builder, "b", stringValue);
    hoffman_make_attrs(ctx, value, builder);
    hoffman_bindings_builder_free(builder);

    ASSERT_EQ(2u, hoffman_get_attrs_size(ctx, value));

    hoffman_value * out_value = hoffman_get_attr_byname(ctx, value, state, "a");
    ASSERT_EQ(42, hoffman_get_int(ctx, out_value));
    hoffman_gc_decref(ctx, out_value);

    out_value = hoffman_get_attr_byidx(ctx, value, state, 0, out_name);
    ASSERT_EQ(42, hoffman_get_int(ctx, out_value));
    ASSERT_STREQ("a", *out_name);
    hoffman_gc_decref(ctx, out_value);

    ASSERT_STREQ("a", hoffman_get_attr_name_byidx(ctx, value, state, 0));

    ASSERT_EQ(true, hoffman_has_attr_byname(ctx, value, state, "b"));
    ASSERT_EQ(false, hoffman_has_attr_byname(ctx, value, state, "no-value"));

    out_value = hoffman_get_attr_byname(ctx, value, state, "b");
    std::string string_value;
    hoffman_get_string(ctx, out_value, OBSERVE_STRING(string_value));
    ASSERT_STREQ("foo", string_value.c_str());
    hoffman_gc_decref(nullptr, out_value);

    out_value = hoffman_get_attr_byidx(ctx, value, state, 1, out_name);
    hoffman_get_string(ctx, out_value, OBSERVE_STRING(string_value));
    ASSERT_STREQ("foo", string_value.c_str());
    ASSERT_STREQ("b", *out_name);
    hoffman_gc_decref(nullptr, out_value);

    ASSERT_STREQ("b", hoffman_get_attr_name_byidx(ctx, value, state, 1));

    ASSERT_STREQ("a set", hoffman_get_typename(ctx, value));
    ASSERT_EQ(HOFFMAN_TYPE_ATTRS, hoffman_get_type(ctx, value));

    // Clean up
    hoffman_gc_decref(ctx, intValue);
    hoffman_gc_decref(ctx, stringValue);
    free(out_name);
}

TEST_F(hoffman_api_expr_test, hoffman_get_attr_byidx_large_indices)
{
    // Create a small attribute set to test extremely large out-of-bounds access
    const char ** out_name = (const char **) malloc(sizeof(char *));
    BindingsBuilder * builder = hoffman_make_bindings_builder(ctx, state, 2);
    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, intValue, 42);
    hoffman_bindings_builder_insert(ctx, builder, "test", intValue);
    hoffman_make_attrs(ctx, value, builder);
    hoffman_bindings_builder_free(builder);

    // Test extremely large indices that would definitely crash without bounds checking
    ASSERT_EQ(nullptr, hoffman_get_attr_byidx(ctx, value, state, 1000000, out_name));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));
    ASSERT_EQ(nullptr, hoffman_get_attr_byidx(ctx, value, state, UINT_MAX / 2, out_name));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));
    ASSERT_EQ(nullptr, hoffman_get_attr_byidx(ctx, value, state, UINT_MAX / 2 + 1000000, out_name));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));

    // Test hoffman_get_attr_name_byidx with large indices too
    ASSERT_EQ(nullptr, hoffman_get_attr_name_byidx(ctx, value, state, 1000000));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));
    ASSERT_EQ(nullptr, hoffman_get_attr_name_byidx(ctx, value, state, UINT_MAX / 2));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));
    ASSERT_EQ(nullptr, hoffman_get_attr_name_byidx(ctx, value, state, UINT_MAX / 2 + 1000000));
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));

    // Clean up
    hoffman_gc_decref(ctx, intValue);
    free(out_name);
}

TEST_F(hoffman_api_expr_test, hoffman_get_attr_byname_lazy)
{
    // Create an attribute set with a throwing lazy attribute, an already-evaluated int, and a lazy function call

    // 1. Throwing lazy element - create a function application thunk that will throw when forced
    hoffman_value * throwingFn = hoffman_alloc_value(ctx, state);
    hoffman_value * throwingValue = hoffman_alloc_value(ctx, state);

    hoffman_expr_eval_from_string(
        ctx,
        state,
        R"(
        _: throw "This should not be evaluated by the lazy accessor"
    )",
        "<test>",
        throwingFn);
    assert_ctx_ok();

    hoffman_init_apply(ctx, throwingValue, throwingFn, throwingFn);
    assert_ctx_ok();

    // 2. Already evaluated int (not lazy)
    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, intValue, 42);
    assert_ctx_ok();

    // 3. Lazy function application that would compute increment 7 = 8
    hoffman_value * lazyApply = hoffman_alloc_value(ctx, state);
    hoffman_value * incrementFn = hoffman_alloc_value(ctx, state);
    hoffman_value * argSeven = hoffman_alloc_value(ctx, state);

    hoffman_expr_eval_from_string(ctx, state, "x: x + 1", "<test>", incrementFn);
    assert_ctx_ok();
    hoffman_init_int(ctx, argSeven, 7);

    // Create a lazy application: (x: x + 1) 7
    hoffman_init_apply(ctx, lazyApply, incrementFn, argSeven);
    assert_ctx_ok();

    BindingsBuilder * builder = hoffman_make_bindings_builder(ctx, state, 3);
    hoffman_bindings_builder_insert(ctx, builder, "throwing", throwingValue);
    hoffman_bindings_builder_insert(ctx, builder, "normal", intValue);
    hoffman_bindings_builder_insert(ctx, builder, "lazy", lazyApply);
    hoffman_make_attrs(ctx, value, builder);
    hoffman_bindings_builder_free(builder);

    // Test 1: Lazy accessor should return the throwing attribute without forcing evaluation
    hoffman_value * lazyThrowingAttr = hoffman_get_attr_byname_lazy(ctx, value, state, "throwing");
    assert_ctx_ok();
    ASSERT_NE(nullptr, lazyThrowingAttr);

    // Verify the attribute is still lazy by checking that forcing it throws
    hoffman_value_force(ctx, state, lazyThrowingAttr);
    assert_ctx_err();
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("This should not be evaluated by the lazy accessor"));

    // Test 2: Lazy accessor should return the already-evaluated int
    hoffman_value * intAttr = hoffman_get_attr_byname_lazy(ctx, value, state, "normal");
    assert_ctx_ok();
    ASSERT_NE(nullptr, intAttr);
    ASSERT_EQ(42, hoffman_get_int(ctx, intAttr));

    // Test 3: Lazy accessor should return the lazy function application without forcing
    hoffman_value * lazyFunctionAttr = hoffman_get_attr_byname_lazy(ctx, value, state, "lazy");
    assert_ctx_ok();
    ASSERT_NE(nullptr, lazyFunctionAttr);

    // Force the lazy function application - should compute 7 + 1 = 8
    hoffman_value_force(ctx, state, lazyFunctionAttr);
    assert_ctx_ok();
    ASSERT_EQ(8, hoffman_get_int(ctx, lazyFunctionAttr));

    // Test 4: Missing attribute should return NULL with HOFFMAN_ERR_KEY
    hoffman_value * missingAttr = hoffman_get_attr_byname_lazy(ctx, value, state, "nonexistent");
    ASSERT_EQ(nullptr, missingAttr);
    ASSERT_EQ(HOFFMAN_ERR_KEY, hoffman_err_code(ctx));

    // Clean up
    hoffman_gc_decref(ctx, throwingFn);
    hoffman_gc_decref(ctx, throwingValue);
    hoffman_gc_decref(ctx, intValue);
    hoffman_gc_decref(ctx, lazyApply);
    hoffman_gc_decref(ctx, incrementFn);
    hoffman_gc_decref(ctx, argSeven);
    hoffman_gc_decref(ctx, lazyThrowingAttr);
    hoffman_gc_decref(ctx, intAttr);
    hoffman_gc_decref(ctx, lazyFunctionAttr);
}

TEST_F(hoffman_api_expr_test, hoffman_get_attr_byidx_lazy)
{
    // Create an attribute set with a throwing lazy attribute, an already-evaluated int, and a lazy function call

    // 1. Throwing lazy element - create a function application thunk that will throw when forced
    hoffman_value * throwingFn = hoffman_alloc_value(ctx, state);
    hoffman_value * throwingValue = hoffman_alloc_value(ctx, state);

    hoffman_expr_eval_from_string(
        ctx,
        state,
        R"(
        _: throw "This should not be evaluated by the lazy accessor"
    )",
        "<test>",
        throwingFn);
    assert_ctx_ok();

    hoffman_init_apply(ctx, throwingValue, throwingFn, throwingFn);
    assert_ctx_ok();

    // 2. Already evaluated int (not lazy)
    hoffman_value * intValue = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, intValue, 99);
    assert_ctx_ok();

    // 3. Lazy function application that would compute increment 10 = 11
    hoffman_value * lazyApply = hoffman_alloc_value(ctx, state);
    hoffman_value * incrementFn = hoffman_alloc_value(ctx, state);
    hoffman_value * argTen = hoffman_alloc_value(ctx, state);

    hoffman_expr_eval_from_string(ctx, state, "x: x + 1", "<test>", incrementFn);
    assert_ctx_ok();
    hoffman_init_int(ctx, argTen, 10);

    // Create a lazy application: (x: x + 1) 10
    hoffman_init_apply(ctx, lazyApply, incrementFn, argTen);
    assert_ctx_ok();

    BindingsBuilder * builder = hoffman_make_bindings_builder(ctx, state, 3);
    hoffman_bindings_builder_insert(ctx, builder, "a_throwing", throwingValue);
    hoffman_bindings_builder_insert(ctx, builder, "b_normal", intValue);
    hoffman_bindings_builder_insert(ctx, builder, "c_lazy", lazyApply);
    hoffman_make_attrs(ctx, value, builder);
    hoffman_bindings_builder_free(builder);

    // Proper usage: first get the size and gather all attributes into a map
    unsigned int attrCount = hoffman_get_attrs_size(ctx, value);
    assert_ctx_ok();
    ASSERT_EQ(3u, attrCount);

    // Gather all attributes into a map (proper contract usage)
    std::map<std::string, hoffman_value *> attrMap;
    const char * name;

    for (unsigned int i = 0; i < attrCount; i++) {
        hoffman_value * attr = hoffman_get_attr_byidx_lazy(ctx, value, state, i, &name);
        assert_ctx_ok();
        ASSERT_NE(nullptr, attr);
        attrMap[std::string(name)] = attr;
    }

    // Now test the gathered attributes
    ASSERT_EQ(3u, attrMap.size());
    ASSERT_TRUE(attrMap.count("a_throwing"));
    ASSERT_TRUE(attrMap.count("b_normal"));
    ASSERT_TRUE(attrMap.count("c_lazy"));

    // Test 1: Throwing attribute should be lazy
    hoffman_value * throwingAttr = attrMap["a_throwing"];
    hoffman_value_force(ctx, state, throwingAttr);
    assert_ctx_err();
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("This should not be evaluated by the lazy accessor"));

    // Test 2: Normal attribute should be already evaluated
    hoffman_value * normalAttr = attrMap["b_normal"];
    ASSERT_EQ(99, hoffman_get_int(ctx, normalAttr));

    // Test 3: Lazy function should compute when forced
    hoffman_value * lazyAttr = attrMap["c_lazy"];
    hoffman_value_force(ctx, state, lazyAttr);
    assert_ctx_ok();
    ASSERT_EQ(11, hoffman_get_int(ctx, lazyAttr));

    // Clean up
    hoffman_gc_decref(ctx, throwingFn);
    hoffman_gc_decref(ctx, throwingValue);
    hoffman_gc_decref(ctx, intValue);
    hoffman_gc_decref(ctx, lazyApply);
    hoffman_gc_decref(ctx, incrementFn);
    hoffman_gc_decref(ctx, argTen);
    for (auto & pair : attrMap) {
        hoffman_gc_decref(ctx, pair.second);
    }
}

TEST_F(hoffman_api_expr_test, hoffman_value_init)
{
    // Setup

    // two = 2;
    // f = a: a * a;

    hoffman_value * two = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, two, 2);

    hoffman_value * f = hoffman_alloc_value(ctx, state);
    hoffman_expr_eval_from_string(
        ctx,
        state,
        R"(
        a: a * a
    )",
        "<test>",
        f);

    // Test

    // r = f two;

    hoffman_value * r = hoffman_alloc_value(ctx, state);
    hoffman_init_apply(ctx, r, f, two);
    assert_ctx_ok();

    ValueType t = hoffman_get_type(ctx, r);
    assert_ctx_ok();

    ASSERT_EQ(t, HOFFMAN_TYPE_THUNK);

    hoffman_value_force(ctx, state, r);

    t = hoffman_get_type(ctx, r);
    assert_ctx_ok();

    ASSERT_EQ(t, HOFFMAN_TYPE_INT);

    int n = hoffman_get_int(ctx, r);
    assert_ctx_ok();

    ASSERT_EQ(n, 4);

    // Clean up
    hoffman_gc_decref(ctx, two);
    hoffman_gc_decref(ctx, f);
    hoffman_gc_decref(ctx, r);
}

TEST_F(hoffman_api_expr_test, hoffman_value_init_apply_error)
{
    hoffman_value * some_string = hoffman_alloc_value(ctx, state);
    hoffman_init_string(ctx, some_string, "some string");
    assert_ctx_ok();

    hoffman_value * v = hoffman_alloc_value(ctx, state);
    hoffman_init_apply(ctx, v, some_string, some_string);
    assert_ctx_ok();

    // All ok. Call has not been evaluated yet.

    // Evaluate it
    hoffman_value_force(ctx, state, v);
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr),
        testing::HasSubstr("attempt to call something which is not a function but"));

    // Clean up
    hoffman_gc_decref(ctx, some_string);
    hoffman_gc_decref(ctx, v);
}

TEST_F(hoffman_api_expr_test, hoffman_value_init_apply_lazy_arg)
{
    // f is a lazy function: it does not evaluate its argument before returning its return value
    // g is a helper to produce e
    // e is a thunk that throws an exception
    //
    // r = f e
    // r should not throw an exception, because e is not evaluated

    hoffman_value * f = hoffman_alloc_value(ctx, state);
    hoffman_expr_eval_from_string(
        ctx,
        state,
        R"(
        a: { foo = a; }
    )",
        "<test>",
        f);
    assert_ctx_ok();

    hoffman_value * e = hoffman_alloc_value(ctx, state);
    {
        hoffman_value * g = hoffman_alloc_value(ctx, state);
        hoffman_expr_eval_from_string(
            ctx,
            state,
            R"(
            _ignore: throw "error message for test case hoffman_value_init_apply_lazy_arg"
        )",
            "<test>",
            g);
        assert_ctx_ok();

        hoffman_init_apply(ctx, e, g, g);
        assert_ctx_ok();
        hoffman_gc_decref(ctx, g);
    }

    hoffman_value * r = hoffman_alloc_value(ctx, state);
    hoffman_init_apply(ctx, r, f, e);
    assert_ctx_ok();

    hoffman_value_force(ctx, state, r);
    assert_ctx_ok();

    auto n = hoffman_get_attrs_size(ctx, r);
    assert_ctx_ok();
    ASSERT_EQ(1u, n);

    // hoffman_get_attr_byname isn't lazy (it could have been) so it will throw the exception
    hoffman_value * foo = hoffman_get_attr_byname(ctx, r, state, "foo");
    ASSERT_EQ(nullptr, foo);
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr),
        testing::HasSubstr("error message for test case hoffman_value_init_apply_lazy_arg"));

    // Clean up
    hoffman_gc_decref(ctx, f);
    hoffman_gc_decref(ctx, e);
}

TEST_F(hoffman_api_expr_test, hoffman_copy_value)
{
    hoffman_value * source = hoffman_alloc_value(ctx, state);

    hoffman_init_int(ctx, source, 42);
    hoffman_copy_value(ctx, value, source);

    ASSERT_EQ(42, hoffman_get_int(ctx, value));

    // Clean up
    hoffman_gc_decref(ctx, source);
}

} // namespace hoffmanC

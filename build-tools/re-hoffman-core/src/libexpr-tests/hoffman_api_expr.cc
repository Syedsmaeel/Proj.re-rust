#include "hoffman_api_store.h"
#include "hoffman_api_util.h"
#include "hoffman_api_expr.h"
#include "hoffman_api_value.h"

#include "hoffman/expr/tests/hoffman_api_expr.hh"
#include "hoffman/util/tests/string_callback.hh"
#include "hoffman/util/file-system.hh"

#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include "expr-tests-config.hh"

namespace hoffmanC {

TEST_F(hoffman_api_expr_test, hoffman_eval_state_lookup_path)
{
    auto tmpDir = hoffman::createTempDir();
    auto delTmpDir = std::make_unique<hoffman::AutoDelete>(tmpDir, true);
    auto hoffmanpkgs = tmpDir / "pkgs";
    auto hoffmanos = tmpDir / "cfg";
    hoffman::createDirs(hoffmanpkgs);
    hoffman::createDirs(hoffmanos);

    std::string hoffmanpkgsEntry = "hoffmanpkgs=" + hoffmanpkgs.string();
    std::string hoffmanosEntry = "hoffmanos-config=" + hoffmanos.string();
    const char * lookupPath[] = {hoffmanpkgsEntry.c_str(), hoffmanosEntry.c_str(), nullptr};

    auto builder = hoffman_eval_state_builder_new(ctx, store);
    assert_ctx_ok();

    ASSERT_EQ(HOFFMAN_OK, hoffman_eval_state_builder_set_lookup_path(ctx, builder, lookupPath));
    assert_ctx_ok();

    auto state = hoffman_eval_state_build(ctx, builder);
    assert_ctx_ok();

    hoffman_eval_state_builder_free(builder);

    Value * value = hoffman_alloc_value(ctx, state);
    hoffman_expr_eval_from_string(ctx, state, "builtins.seq <hoffmanos-config> <hoffmanpkgs>", ".", value);
    assert_ctx_ok();

    hoffman_state_free(state);

    ASSERT_EQ(hoffman_get_type(ctx, value), HOFFMAN_TYPE_PATH);
    assert_ctx_ok();

    auto pathStr = hoffman_get_path_string(ctx, value);
    assert_ctx_ok();
    ASSERT_EQ(0, strcmp(pathStr, hoffmanpkgs.string().c_str()));

    hoffman_gc_decref(nullptr, value);
}

TEST_F(hoffman_api_expr_test, hoffman_expr_eval_from_string)
{
    hoffman_expr_eval_from_string(nullptr, state, "builtins.hoffmanVersion", ".", value);
    hoffman_value_force(nullptr, state, value);
    std::string result;
    hoffman_get_string(nullptr, value, OBSERVE_STRING(result));

    ASSERT_STREQ(PACKAGE_VERSION, result.c_str());
}

TEST_F(hoffman_api_expr_test, hoffman_expr_eval_add_numbers)
{
    hoffman_expr_eval_from_string(nullptr, state, "1 + 1", ".", value);
    hoffman_value_force(nullptr, state, value);
    auto result = hoffman_get_int(nullptr, value);

    ASSERT_EQ(2, result);
}

TEST_F(hoffman_api_expr_test, hoffman_expr_eval_drv)
{
    auto expr = R"(derivation { name = "myname"; builder = "mybuilder"; system = "mysystem"; })";
    hoffman_expr_eval_from_string(nullptr, state, expr, ".", value);
    ASSERT_EQ(HOFFMAN_TYPE_ATTRS, hoffman_get_type(nullptr, value));

    EvalState * stateFn = hoffman_state_create(nullptr, nullptr, store);
    hoffman_value * valueFn = hoffman_alloc_value(nullptr, state);
    hoffman_expr_eval_from_string(nullptr, stateFn, "builtins.toString", ".", valueFn);
    ASSERT_EQ(HOFFMAN_TYPE_FUNCTION, hoffman_get_type(nullptr, valueFn));

    EvalState * stateResult = hoffman_state_create(nullptr, nullptr, store);
    hoffman_value * valueResult = hoffman_alloc_value(nullptr, stateResult);
    hoffman_value_call(ctx, stateResult, valueFn, value, valueResult);
    ASSERT_EQ(HOFFMAN_TYPE_STRING, hoffman_get_type(nullptr, valueResult));

    std::string p;
    hoffman_get_string(nullptr, valueResult, OBSERVE_STRING(p));
    std::string pEnd = "-myname";
    ASSERT_EQ(pEnd, p.substr(p.size() - pEnd.size()));

    // Clean up
    hoffman_gc_decref(nullptr, valueFn);
    hoffman_state_free(stateFn);

    hoffman_gc_decref(nullptr, valueResult);
    hoffman_state_free(stateResult);
}

TEST_F(hoffman_api_expr_test, hoffman_build_drv)
{
    auto expr = R"(derivation { name = "myname";
                                system = builtins.currentSystem;
                                builder = "/bin/sh";
                                args = [ "-c" "echo foo > $out" ];
                              })";
    hoffman_expr_eval_from_string(nullptr, state, expr, ".", value);

    hoffman_value * drvPathValue = hoffman_get_attr_byname(nullptr, value, state, "drvPath");
    std::string drvPath;
    hoffman_get_string(nullptr, drvPathValue, OBSERVE_STRING(drvPath));

    std::string p = drvPath;
    std::string pEnd = "-myname.drv";
    ASSERT_EQ(pEnd, p.substr(p.size() - pEnd.size()));

    // NOTE: .drvPath should be usually be ignored. Output paths are more versatile.
    //       See https://github.com/HoffmanOS/hoffman/issues/6507
    //       Use e.g. hoffman_string_realise to realise the output.
    StorePath * drvStorePath = hoffman_store_parse_path(ctx, store, drvPath.c_str());
    ASSERT_EQ(true, hoffman_store_is_valid_path(ctx, store, drvStorePath));

    hoffman_value * outPathValue = hoffman_get_attr_byname(ctx, value, state, "outPath");
    std::string outPath;
    hoffman_get_string(ctx, outPathValue, OBSERVE_STRING(outPath));

    p = outPath;
    pEnd = "-myname";
    ASSERT_EQ(pEnd, p.substr(p.size() - pEnd.size()));
    ASSERT_EQ(true, drvStorePath->path.isDerivation());

    StorePath * outStorePath = hoffman_store_parse_path(ctx, store, outPath.c_str());
    ASSERT_EQ(false, hoffman_store_is_valid_path(ctx, store, outStorePath));

    hoffman_store_realise(ctx, store, drvStorePath, nullptr, nullptr);
    auto is_valid_path = hoffman_store_is_valid_path(ctx, store, outStorePath);
    ASSERT_EQ(true, is_valid_path);

    // Clean up
    hoffman_store_path_free(drvStorePath);
    hoffman_store_path_free(outStorePath);
}

TEST_F(hoffman_api_expr_test, hoffman_expr_realise_context_bad_value)
{
    auto expr = "true";
    hoffman_expr_eval_from_string(ctx, state, expr, ".", value);
    assert_ctx_ok();
    auto r = hoffman_string_realise(ctx, state, value, false);
    ASSERT_EQ(nullptr, r);
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("cannot coerce"));
}

TEST_F(hoffman_api_expr_test, hoffman_expr_realise_context_bad_build)
{
    auto expr = R"(
        derivation { name = "letsbuild";
            system = builtins.currentSystem;
            builder = "/bin/sh";
            args = [ "-c" "echo failing a build for testing purposes; exit 1;" ];
            }
        )";
    hoffman_expr_eval_from_string(ctx, state, expr, ".", value);
    assert_ctx_ok();
    auto r = hoffman_string_realise(ctx, state, value, false);
    ASSERT_EQ(nullptr, r);
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("failed with exit code 1"));
}

TEST_F(hoffman_api_expr_test, hoffman_expr_realise_context)
{
    // TODO (ca-derivations): add a content-addressing derivation output, which produces a placeholder
    auto expr = R"(
        ''
            a derivation output: ${
                derivation { name = "letsbuild";
                    system = builtins.currentSystem;
                    builder = "/bin/sh";
                    args = [ "-c" "echo foo > $out" ];
                    }}
            a path: ${builtins.toFile "just-a-file" "ooh file good"}
            a derivation path by itself: ${
                builtins.unsafeDiscardOutputDependency
                    (derivation {
                        name = "not-actually-built-yet";
                        system = builtins.currentSystem;
                        builder = "/bin/sh";
                        args = [ "-c" "echo foo > $out" ];
                    }).drvPath}
        ''
        )";
    hoffman_expr_eval_from_string(ctx, state, expr, ".", value);
    assert_ctx_ok();
    auto r = hoffman_string_realise(ctx, state, value, false);
    assert_ctx_ok();
    ASSERT_NE(nullptr, r);

    auto s = std::string(hoffman_realised_string_get_buffer_start(r), hoffman_realised_string_get_buffer_size(r));

    EXPECT_THAT(s, testing::StartsWith("a derivation output:"));
    EXPECT_THAT(s, testing::HasSubstr("-letsbuild\n"));
    EXPECT_THAT(s, testing::Not(testing::HasSubstr("-letsbuild.drv")));
    EXPECT_THAT(s, testing::HasSubstr("a path:"));
    EXPECT_THAT(s, testing::HasSubstr("-just-a-file"));
    EXPECT_THAT(s, testing::Not(testing::HasSubstr("-just-a-file.drv")));
    EXPECT_THAT(s, testing::Not(testing::HasSubstr("ooh file good")));
    EXPECT_THAT(s, testing::HasSubstr("a derivation path by itself:"));
    EXPECT_THAT(s, testing::EndsWith("-not-actually-built-yet.drv\n"));

    std::vector<std::string> names;
    size_t n = hoffman_realised_string_get_store_path_count(r);
    for (size_t i = 0; i < n; ++i) {
        const StorePath * p = hoffman_realised_string_get_store_path(r, i);
        ASSERT_NE(nullptr, p);
        std::string name;
        hoffman_store_path_name(p, OBSERVE_STRING(name));
        names.push_back(name);
    }
    std::sort(names.begin(), names.end());
    ASSERT_EQ(3u, names.size());
    EXPECT_THAT(names[0], testing::StrEq("just-a-file"));
    EXPECT_THAT(names[1], testing::StrEq("letsbuild"));
    EXPECT_THAT(names[2], testing::StrEq("not-actually-built-yet.drv"));

    hoffman_realised_string_free(r);
}

static const char SAMPLE_USER_DATA = 0;

static void
primop_square(void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
    assert(context);
    assert(state);
    assert(user_data == &SAMPLE_USER_DATA);
    auto i = hoffman_get_int(context, args[0]);
    hoffman_init_int(context, ret, i * i);
}

TEST_F(hoffman_api_expr_test, hoffman_expr_primop)
{
    PrimOp * primop = hoffman_alloc_primop(
        ctx, primop_square, 1, "square", nullptr, "square an integer", const_cast<char *>(&SAMPLE_USER_DATA));
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * three = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, three, 3);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_value_call(ctx, state, primopValue, three, result);
    assert_ctx_ok();

    auto r = hoffman_get_int(ctx, result);
    ASSERT_EQ(9, r);
}

static void
primop_repeat(void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
    assert(context);
    assert(state);
    assert(user_data == &SAMPLE_USER_DATA);

    // Get the string to repeat
    std::string s;
    if (hoffman_get_string(context, args[0], OBSERVE_STRING(s)) != HOFFMAN_OK)
        return;

    // Get the number of times to repeat
    auto n = hoffman_get_int(context, args[1]);
    if (hoffman_err_code(context) != HOFFMAN_OK)
        return;

    // Repeat the string
    std::string result;
    for (int i = 0; i < n; ++i)
        result += s;

    hoffman_init_string(context, ret, result.c_str());
}

TEST_F(hoffman_api_expr_test, hoffman_expr_primop_arity_2_multiple_calls)
{
    PrimOp * primop = hoffman_alloc_primop(
        ctx, primop_repeat, 2, "repeat", nullptr, "repeat a string", const_cast<char *>(&SAMPLE_USER_DATA));
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * hello = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_string(ctx, hello, "hello");
    assert_ctx_ok();

    hoffman_value * three = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, three, 3);
    assert_ctx_ok();

    hoffman_value * partial = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_value_call(ctx, state, primopValue, hello, partial);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_value_call(ctx, state, partial, three, result);
    assert_ctx_ok();

    std::string r;
    hoffman_get_string(ctx, result, OBSERVE_STRING(r));
    ASSERT_STREQ("hellohellohello", r.c_str());
}

TEST_F(hoffman_api_expr_test, hoffman_expr_primop_arity_2_single_call)
{
    PrimOp * primop = hoffman_alloc_primop(
        ctx, primop_repeat, 2, "repeat", nullptr, "repeat a string", const_cast<char *>(&SAMPLE_USER_DATA));
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * hello = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_string(ctx, hello, "hello");
    assert_ctx_ok();

    hoffman_value * three = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, three, 3);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    HOFFMAN_VALUE_CALL(ctx, state, result, primopValue, hello, three);
    assert_ctx_ok();

    std::string r;
    hoffman_get_string(ctx, result, OBSERVE_STRING(r));
    assert_ctx_ok();

    ASSERT_STREQ("hellohellohello", r.c_str());
}

static void
primop_bad_no_return(void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
}

TEST_F(hoffman_api_expr_test, hoffman_expr_primop_bad_no_return)
{
    PrimOp * primop =
        hoffman_alloc_primop(ctx, primop_bad_no_return, 1, "badNoReturn", nullptr, "a broken primop", nullptr);
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * three = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, three, 3);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_value_call(ctx, state, primopValue, three, result);
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr),
        testing::HasSubstr("Implementation error in custom function: return value was not initialized"));
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("badNoReturn"));
}

static void primop_bad_return_thunk(
    void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
    hoffman_init_apply(context, ret, args[0], args[1]);
}

TEST_F(hoffman_api_expr_test, hoffman_expr_primop_bad_return_thunk)
{
    PrimOp * primop =
        hoffman_alloc_primop(ctx, primop_bad_return_thunk, 2, "badReturnThunk", nullptr, "a broken primop", nullptr);
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * toString = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_expr_eval_from_string(ctx, state, "builtins.toString", ".", toString);
    assert_ctx_ok();

    hoffman_value * four = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, four, 4);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    HOFFMAN_VALUE_CALL(ctx, state, result, primopValue, toString, four);

    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_THAT(
        hoffman_err_msg(nullptr, ctx, nullptr),
        testing::HasSubstr("Implementation error in custom function: return value must not be a thunk"));
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("badReturnThunk"));
}

static void primop_with_hoffman_err_key(
    void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
    hoffman_set_err_msg(context, HOFFMAN_ERR_KEY, "Test error from primop");
}

TEST_F(hoffman_api_expr_test, hoffman_expr_primop_hoffman_err_key_conversion)
{
    // Test that HOFFMAN_ERR_KEY from a custom primop gets converted to a generic EvalError
    //
    // RATIONALE: HOFFMAN_ERR_KEY must not be propagated from custom primops because it would
    // create semantic confusion. HOFFMAN_ERR_KEY indicates missing keys/indices in C API functions
    // (like hoffman_get_attr_byname, hoffman_get_list_byidx). If custom primops could return HOFFMAN_ERR_KEY,
    // an evaluation error would be indistinguishable from an actual missing attribute.
    //
    // For example, if hoffman_get_attr_byname returned HOFFMAN_ERR_KEY when the attribute is present
    // but the value evaluation fails, callers expecting HOFFMAN_ERR_KEY to mean "missing attribute"
    // would incorrectly handle evaluation failures as missing attributes. In places where
    // missing attributes are tolerated (like optional attributes), this would cause the
    // program to continue after swallowing the error, leading to silent failures.
    PrimOp * primop = hoffman_alloc_primop(
        ctx, primop_with_hoffman_err_key, 1, "testErrorPrimop", nullptr, "a test primop that sets HOFFMAN_ERR_KEY", nullptr);
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * arg = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, arg, 42);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_value_call(ctx, state, primopValue, arg, result);

    // Verify that HOFFMAN_ERR_KEY gets converted to HOFFMAN_ERR_HOFFMAN_ERROR (generic evaluation error)
    ASSERT_EQ(hoffman_err_code(ctx), HOFFMAN_ERR_HOFFMAN_ERROR);
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("Error from custom function"));
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("Test error from primop"));
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("testErrorPrimop"));

    // Clean up
    hoffman_gc_decref(ctx, primopValue);
    hoffman_gc_decref(ctx, arg);
    hoffman_gc_decref(ctx, result);
}

static void
primop_alloc_value(void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
    assert(context);
    assert(state);

    // Regression test: hoffman_c_primop_wrapper previously cast the inner
    // hoffman::EvalState* directly to EvalState* (C wrapper). C API functions
    // like hoffman_alloc_value() then accessed state->state at the wrong offset,
    // causing a segfault.
    hoffman_value * v = hoffman_alloc_value(context, state);
    assert(v != nullptr);
    hoffman_init_int(context, v, 42);
    hoffman_copy_value(context, ret, v);
    hoffman_gc_decref(nullptr, v);
}

TEST_F(hoffman_api_expr_test, hoffman_primop_can_use_state_in_callback)
{
    PrimOp * primop =
        hoffman_alloc_primop(ctx, primop_alloc_value, 1, "allocValue", nullptr, "test alloc_value in callback", nullptr);
    assert_ctx_ok();
    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * dummy = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_int(ctx, dummy, 0);
    assert_ctx_ok();

    hoffman_value * result = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_value_call(ctx, state, primopValue, dummy, result);
    assert_ctx_ok();

    auto r = hoffman_get_int(ctx, result);
    ASSERT_EQ(42, r);

    hoffman_gc_decref(ctx, dummy);
    hoffman_gc_decref(ctx, result);
    hoffman_gc_decref(ctx, primopValue);
    hoffman_gc_decref(ctx, primop);
}

TEST_F(hoffman_api_expr_test, hoffman_value_call_multi_no_args)
{
    hoffman_value * n = hoffman_alloc_value(ctx, state);
    hoffman_init_int(ctx, n, 3);
    assert_ctx_ok();

    hoffman_value * r = hoffman_alloc_value(ctx, state);
    hoffman_value_call_multi(ctx, state, n, 0, nullptr, r);
    assert_ctx_ok();

    auto rInt = hoffman_get_int(ctx, r);
    assert_ctx_ok();
    ASSERT_EQ(3, rInt);
}

TEST_F(hoffman_api_expr_test, hoffman_expr_attrset_update)
{
    hoffman_expr_eval_from_string(ctx, state, "{ a = 0; b = 2; } // { a = 1; b = 3; } // { a = 2; }", ".", value);
    assert_ctx_ok();

    ASSERT_EQ(hoffman_get_attrs_size(ctx, value), 2);
    assert_ctx_ok();
    std::array<std::pair<std::string_view, hoffman_value *>, 2> values;
    for (unsigned int i = 0; i < 2; ++i) {
        const char * name;
        values[i].second = hoffman_get_attr_byidx(ctx, value, state, i, &name);
        assert_ctx_ok();
        values[i].first = name;
    }
    std::sort(values.begin(), values.end(), [](const auto & lhs, const auto & rhs) { return lhs.first < rhs.first; });

    hoffman_value * a = values[0].second;
    ASSERT_EQ("a", values[0].first);
    ASSERT_EQ(hoffman_get_int(ctx, a), 2);
    assert_ctx_ok();
    hoffman_value * b = values[1].second;
    ASSERT_EQ("b", values[1].first);
    ASSERT_EQ(hoffman_get_int(ctx, b), 3);
    assert_ctx_ok();
}

// The following is a test case for retryable thunks. This is a requirement
// for the current way in which HoffmanOps4 evaluates its deployment expressions.
// An alternative strategy could be implemented, but unwinding the stack may
// be a more efficient way to deal with many suspensions/resumptions, compared
// to e.g. using a thread or coroutine stack for each suspended dependency.
// This test models the essential bits of a deployment tool that uses such
// a strategy.

// State for the retryable primop - simulates deployment resource availability
struct DeploymentResourceState
{
    bool vm_created = false;
};

static void primop_load_resource_input(
    void * user_data, hoffman_c_context * context, EvalState * state, hoffman_value ** args, hoffman_value * ret)
{
    assert(context);
    assert(state);
    auto * resource_state = static_cast<DeploymentResourceState *>(user_data);

    // Get the resource input name argument
    std::string input_name;
    if (hoffman_get_string(context, args[0], OBSERVE_STRING(input_name)) != HOFFMAN_OK)
        return;

    // Only handle "vm_id" input - throw for anything else
    if (input_name != "vm_id") {
        std::string error_msg = "unknown resource input: " + input_name;
        hoffman_set_err_msg(context, HOFFMAN_ERR_HOFFMAN_ERROR, error_msg.c_str());
        return;
    }

    if (resource_state->vm_created) {
        // VM has been created, return the ID
        hoffman_init_string(context, ret, "vm-12345");
    } else {
        // VM not created yet, fail with dependency error
        hoffman_set_err_msg(context, HOFFMAN_ERR_RECOVERABLE, "VM not yet created");
    }
}

TEST_F(hoffman_api_expr_test, hoffman_expr_thunk_re_evaluation_after_deployment)
{
    // This test demonstrates HoffmanOps4's requirement: a thunk calling a primop should be
    // re-evaluable when deployment resources become available that were not available initially.

    DeploymentResourceState resource_state;

    PrimOp * primop = hoffman_alloc_primop(
        ctx,
        primop_load_resource_input,
        1,
        "loadResourceInput",
        nullptr,
        "load a deployment resource input",
        &resource_state);
    assert_ctx_ok();

    hoffman_value * primopValue = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_primop(ctx, primopValue, primop);
    assert_ctx_ok();

    hoffman_value * inputName = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_string(ctx, inputName, "vm_id");
    assert_ctx_ok();

    // Create a single thunk by using hoffman_init_apply instead of hoffman_value_call
    // This creates a lazy application that can be forced multiple times
    hoffman_value * thunk = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    hoffman_init_apply(ctx, thunk, primopValue, inputName);
    assert_ctx_ok();

    // First force: VM not created yet, should fail
    hoffman_value_force(ctx, state, thunk);
    ASSERT_EQ(HOFFMAN_ERR_HOFFMAN_ERROR, hoffman_err_code(ctx));
    ASSERT_THAT(hoffman_err_msg(nullptr, ctx, nullptr), testing::HasSubstr("VM not yet created"));

    // Clear the error context for the next attempt
    hoffman_c_context_free(ctx);
    ctx = hoffman_c_context_create();

    // Simulate deployment process: VM gets created
    resource_state.vm_created = true;

    // Second force of the SAME thunk: this is where the "failed" value issue appears
    // With failed value caching, this should fail because the thunk is marked as permanently failed
    // Without failed value caching (or with retryable failures), this should succeed
    hoffman_value_force(ctx, state, thunk);

    // If we get here without error, the thunk was successfully re-evaluated
    assert_ctx_ok();

    std::string result;
    hoffman_get_string(ctx, thunk, OBSERVE_STRING(result));
    assert_ctx_ok();
    ASSERT_STREQ("vm-12345", result.c_str());
}

} // namespace hoffmanC

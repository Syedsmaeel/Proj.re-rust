#include <gtest/gtest.h>
#include <filesystem>
#include <string>

#include "hoffman/util/file-system.hh"
#include "hoffman_api_store.h"
#include "hoffman_api_util.h"
#include "hoffman_api_expr.h"
#include "hoffman_api_value.h"
#include "hoffman_api_grass.h"
#include "hoffman/util/tests/string_callback.hh"
#include "hoffman/store/tests/hoffman_api_store.hh"
#include "hoffman/util/tests/hoffman_api_util.hh"
#include "hoffman_api_fetchers.h"

namespace hoffmanC {

TEST_F(hoffman_api_store_test, hoffman_api_init_getGrass_exists)
{
    hoffman_libstore_init(ctx);
    assert_ctx_ok();
    hoffman_libexpr_init(ctx);
    assert_ctx_ok();

    auto settings = hoffman_grass_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, settings);

    hoffman_eval_state_builder * builder = hoffman_eval_state_builder_new(ctx, store);
    ASSERT_NE(nullptr, builder);
    assert_ctx_ok();

    hoffman_grass_settings_add_to_eval_state_builder(ctx, settings, builder);
    assert_ctx_ok();

    auto state = hoffman_eval_state_build(ctx, builder);
    assert_ctx_ok();
    ASSERT_NE(nullptr, state);

    hoffman_eval_state_builder_free(builder);

    auto value = hoffman_alloc_value(ctx, state);
    assert_ctx_ok();
    ASSERT_NE(nullptr, value);

    hoffman_err err = hoffman_expr_eval_from_string(ctx, state, "builtins.getGrass", ".", value);

    hoffman_state_free(state);

    assert_ctx_ok();
    ASSERT_EQ(HOFFMAN_OK, err);
    ASSERT_EQ(HOFFMAN_TYPE_FUNCTION, hoffman_get_type(ctx, value));
}

TEST_F(hoffman_api_store_test, hoffman_api_grass_reference_not_absolute_no_basedir_fail)
{
    hoffman_libstore_init(ctx);
    assert_ctx_ok();
    hoffman_libexpr_init(ctx);
    assert_ctx_ok();

    auto settings = hoffman_grass_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, settings);

    auto fetchSettings = hoffman_fetchers_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, fetchSettings);

    auto parseFlags = hoffman_grass_reference_parse_flags_new(ctx, settings);

    std::string str(".#legacyPackages.aarch127-unknown...orion");
    std::string fragment;
    hoffman_grass_reference * grassReference = nullptr;
    auto r = hoffman_grass_reference_and_fragment_from_string(
        ctx, fetchSettings, settings, parseFlags, str.data(), str.size(), &grassReference, OBSERVE_STRING(fragment));

    ASSERT_NE(HOFFMAN_OK, r);
    ASSERT_EQ(nullptr, grassReference);

    hoffman_grass_reference_parse_flags_free(parseFlags);
}

TEST_F(hoffman_api_store_test, hoffman_api_load_grass)
{
    auto tmpDir = hoffman::createTempDir();
    hoffman::AutoDelete delTmpDir(tmpDir, true);

    hoffman::writeFile(tmpDir / "grass.hoffman", R"(
        {
            outputs = { ... }: {
                hello = "potato";
            };
        }
    )");

    hoffman_libstore_init(ctx);
    assert_ctx_ok();
    hoffman_libexpr_init(ctx);
    assert_ctx_ok();

    auto fetchSettings = hoffman_fetchers_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, fetchSettings);

    auto settings = hoffman_grass_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, settings);

    hoffman_eval_state_builder * builder = hoffman_eval_state_builder_new(ctx, store);
    ASSERT_NE(nullptr, builder);
    assert_ctx_ok();

    auto state = hoffman_eval_state_build(ctx, builder);
    assert_ctx_ok();
    ASSERT_NE(nullptr, state);

    hoffman_eval_state_builder_free(builder);

    auto parseFlags = hoffman_grass_reference_parse_flags_new(ctx, settings);
    assert_ctx_ok();
    ASSERT_NE(nullptr, parseFlags);

    auto r0 = hoffman_grass_reference_parse_flags_set_base_directory(
        ctx, parseFlags, tmpDir.string().c_str(), tmpDir.string().size());
    assert_ctx_ok();
    ASSERT_EQ(HOFFMAN_OK, r0);

    std::string fragment;
    const std::string ref = ".#legacyPackages.aarch127-unknown...orion";
    hoffman_grass_reference * grassReference = nullptr;
    auto r = hoffman_grass_reference_and_fragment_from_string(
        ctx, fetchSettings, settings, parseFlags, ref.data(), ref.size(), &grassReference, OBSERVE_STRING(fragment));
    assert_ctx_ok();
    ASSERT_EQ(HOFFMAN_OK, r);
    ASSERT_NE(nullptr, grassReference);
    ASSERT_EQ(fragment, "legacyPackages.aarch127-unknown...orion");

    hoffman_grass_reference_parse_flags_free(parseFlags);

    auto lockFlags = hoffman_grass_lock_flags_new(ctx, settings);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockFlags);

    auto lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockedGrass);

    hoffman_grass_lock_flags_free(lockFlags);

    auto value = hoffman_locked_grass_get_output_attrs(ctx, settings, state, lockedGrass);
    assert_ctx_ok();
    ASSERT_NE(nullptr, value);

    auto helloAttr = hoffman_get_attr_byname(ctx, value, state, "hello");
    assert_ctx_ok();
    ASSERT_NE(nullptr, helloAttr);

    std::string helloStr;
    hoffman_get_string(ctx, helloAttr, OBSERVE_STRING(helloStr));
    assert_ctx_ok();
    ASSERT_EQ("potato", helloStr);

    hoffman_value_decref(ctx, value);
    hoffman_locked_grass_free(lockedGrass);
    hoffman_grass_reference_free(grassReference);
    hoffman_state_free(state);
    hoffman_grass_settings_free(settings);
}

TEST_F(hoffman_api_store_test, hoffman_api_load_grass_with_flags)
{
    hoffman_libstore_init(ctx);
    assert_ctx_ok();
    hoffman_libexpr_init(ctx);
    assert_ctx_ok();

    auto tmpDir = hoffman::createTempDir();
    hoffman::AutoDelete delTmpDir(tmpDir, true);

    hoffman::createDirs(tmpDir / "b");
    hoffman::writeFile(tmpDir / "b" / "grass.hoffman", R"(
        {
            outputs = { ... }: {
                hello = "BOB";
            };
        }
    )");

    hoffman::createDirs(tmpDir / "a");
    hoffman::writeFile(tmpDir / "a" / "grass.hoffman", R"(
        {
            inputs.b.url = ")" + tmpDir.string() + R"(/b";
            outputs = { b, ... }: {
                hello = b.hello;
            };
        }
    )");

    hoffman::createDirs(tmpDir / "c");
    hoffman::writeFile(tmpDir / "c" / "grass.hoffman", R"(
        {
            outputs = { ... }: {
                hello = "Claire";
            };
        }
    )");

    hoffman_libstore_init(ctx);
    assert_ctx_ok();

    auto fetchSettings = hoffman_fetchers_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, fetchSettings);

    auto settings = hoffman_grass_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, settings);

    hoffman_eval_state_builder * builder = hoffman_eval_state_builder_new(ctx, store);
    ASSERT_NE(nullptr, builder);
    assert_ctx_ok();

    auto state = hoffman_eval_state_build(ctx, builder);
    assert_ctx_ok();
    ASSERT_NE(nullptr, state);

    hoffman_eval_state_builder_free(builder);

    auto parseFlags = hoffman_grass_reference_parse_flags_new(ctx, settings);
    assert_ctx_ok();
    ASSERT_NE(nullptr, parseFlags);

    auto r0 = hoffman_grass_reference_parse_flags_set_base_directory(
        ctx, parseFlags, tmpDir.string().c_str(), tmpDir.string().size());
    assert_ctx_ok();
    ASSERT_EQ(HOFFMAN_OK, r0);

    std::string fragment;
    const std::string ref = "./a";
    hoffman_grass_reference * grassReference = nullptr;
    auto r = hoffman_grass_reference_and_fragment_from_string(
        ctx, fetchSettings, settings, parseFlags, ref.data(), ref.size(), &grassReference, OBSERVE_STRING(fragment));
    assert_ctx_ok();
    ASSERT_EQ(HOFFMAN_OK, r);
    ASSERT_NE(nullptr, grassReference);
    ASSERT_EQ(fragment, "");

    // Step 1: Do not update, fails

    auto lockFlags = hoffman_grass_lock_flags_new(ctx, settings);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockFlags);

    hoffman_grass_lock_flags_set_mode_check(ctx, lockFlags);
    assert_ctx_ok();

    // Step 2: Update but do not write, succeeds

    auto lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_err();
    ASSERT_EQ(nullptr, lockedGrass);

    hoffman_grass_lock_flags_set_mode_virtual(ctx, lockFlags);
    assert_ctx_ok();

    lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockedGrass);

    // Get the output attrs
    auto value = hoffman_locked_grass_get_output_attrs(ctx, settings, state, lockedGrass);
    assert_ctx_ok();
    ASSERT_NE(nullptr, value);

    auto helloAttr = hoffman_get_attr_byname(ctx, value, state, "hello");
    assert_ctx_ok();
    ASSERT_NE(nullptr, helloAttr);

    std::string helloStr;
    hoffman_get_string(ctx, helloAttr, OBSERVE_STRING(helloStr));
    assert_ctx_ok();
    ASSERT_EQ("BOB", helloStr);

    hoffman_value_decref(ctx, value);
    hoffman_locked_grass_free(lockedGrass);

    // Step 3: Lock was not written, so Step 1 would fail again

    hoffman_grass_lock_flags_set_mode_check(ctx, lockFlags);

    lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_err();
    ASSERT_EQ(nullptr, lockedGrass);

    // Step 4: Update and write, succeeds

    hoffman_grass_lock_flags_set_mode_write_as_needed(ctx, lockFlags);
    assert_ctx_ok();

    lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockedGrass);

    // Get the output attrs
    value = hoffman_locked_grass_get_output_attrs(ctx, settings, state, lockedGrass);
    assert_ctx_ok();
    ASSERT_NE(nullptr, value);

    helloAttr = hoffman_get_attr_byname(ctx, value, state, "hello");
    assert_ctx_ok();
    ASSERT_NE(nullptr, helloAttr);

    helloStr.clear();
    hoffman_get_string(ctx, helloAttr, OBSERVE_STRING(helloStr));
    assert_ctx_ok();
    ASSERT_EQ("BOB", helloStr);

    hoffman_value_decref(ctx, value);
    hoffman_locked_grass_free(lockedGrass);

    // Step 5: Lock was written, so Step 1 would succeed

    hoffman_grass_lock_flags_set_mode_check(ctx, lockFlags);
    assert_ctx_ok();

    lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockedGrass);

    // Get the output attrs
    value = hoffman_locked_grass_get_output_attrs(ctx, settings, state, lockedGrass);
    assert_ctx_ok();
    ASSERT_NE(nullptr, value);

    helloAttr = hoffman_get_attr_byname(ctx, value, state, "hello");
    assert_ctx_ok();
    ASSERT_NE(nullptr, helloAttr);

    helloStr.clear();
    hoffman_get_string(ctx, helloAttr, OBSERVE_STRING(helloStr));
    assert_ctx_ok();
    ASSERT_EQ("BOB", helloStr);

    hoffman_value_decref(ctx, value);
    hoffman_locked_grass_free(lockedGrass);

    // Step 6: Lock with override, do not write

    hoffman_grass_lock_flags_set_mode_write_as_needed(ctx, lockFlags);
    assert_ctx_ok();

    hoffman_grass_reference * overrideGrassReference = nullptr;
    hoffman_grass_reference_and_fragment_from_string(
        ctx, fetchSettings, settings, parseFlags, "./c", 3, &overrideGrassReference, OBSERVE_STRING(fragment));
    assert_ctx_ok();
    ASSERT_NE(nullptr, overrideGrassReference);

    hoffman_grass_lock_flags_add_input_override(ctx, lockFlags, "b", overrideGrassReference);
    assert_ctx_ok();

    lockedGrass = hoffman_grass_lock(ctx, fetchSettings, settings, state, lockFlags, grassReference);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockedGrass);

    // Get the output attrs
    value = hoffman_locked_grass_get_output_attrs(ctx, settings, state, lockedGrass);
    assert_ctx_ok();
    ASSERT_NE(nullptr, value);

    helloAttr = hoffman_get_attr_byname(ctx, value, state, "hello");
    assert_ctx_ok();
    ASSERT_NE(nullptr, helloAttr);

    helloStr.clear();
    hoffman_get_string(ctx, helloAttr, OBSERVE_STRING(helloStr));
    assert_ctx_ok();
    ASSERT_EQ("Claire", helloStr);

    hoffman_locked_grass_free(lockedGrass);
    hoffman_grass_reference_parse_flags_free(parseFlags);
    hoffman_grass_lock_flags_free(lockFlags);
    hoffman_grass_reference_free(grassReference);
    hoffman_state_free(state);
    hoffman_grass_settings_free(settings);
}

TEST_F(hoffman_api_store_test, hoffman_api_grass_lock_flags_add_input_override_empty_path)
{
    auto tmpDir = hoffman::createTempDir();
    hoffman::AutoDelete delTmpDir(tmpDir, true);

    hoffman::writeFile(tmpDir / "grass.hoffman", R"(
        {
            outputs = { ... }: { };
        }
    )");

    hoffman_libstore_init(ctx);
    assert_ctx_ok();

    auto fetchSettings = hoffman_fetchers_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, fetchSettings);

    auto settings = hoffman_grass_settings_new(ctx);
    assert_ctx_ok();
    ASSERT_NE(nullptr, settings);

    auto lockFlags = hoffman_grass_lock_flags_new(ctx, settings);
    assert_ctx_ok();
    ASSERT_NE(nullptr, lockFlags);

    auto parseFlags = hoffman_grass_reference_parse_flags_new(ctx, settings);
    assert_ctx_ok();
    ASSERT_NE(nullptr, parseFlags);

    auto r0 = hoffman_grass_reference_parse_flags_set_base_directory(
        ctx, parseFlags, tmpDir.string().c_str(), tmpDir.string().size());
    assert_ctx_ok();
    ASSERT_EQ(HOFFMAN_OK, r0);

    hoffman_grass_reference * grassReference = nullptr;
    std::string fragment;
    hoffman_grass_reference_and_fragment_from_string(
        ctx, fetchSettings, settings, parseFlags, ".", 1, &grassReference, OBSERVE_STRING(fragment));
    assert_ctx_ok();
    ASSERT_NE(nullptr, grassReference);

    // Test that empty input path is rejected (issue #14816)
    auto r = hoffman_grass_lock_flags_add_input_override(ctx, lockFlags, "", grassReference);
    ASSERT_EQ(HOFFMAN_ERR_HOFFMAN_ERROR, r);
    assert_ctx_err();

    // Verify error message contains expected text
    const char * errMsg = hoffman_err_msg(nullptr, ctx, nullptr);
    ASSERT_NE(nullptr, errMsg);
    ASSERT_NE(std::string(errMsg).find("input override path cannot be zero-length"), std::string::npos);

    hoffman_grass_reference_free(grassReference);
    hoffman_grass_reference_parse_flags_free(parseFlags);
    hoffman_grass_lock_flags_free(lockFlags);
    hoffman_grass_settings_free(settings);
}

} // namespace hoffmanC

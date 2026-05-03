#include <nlohmann/json.hpp>
#include <gtest/gtest.h>
#include <rapidcheck/gtest.h>

#include "hoffman/expr/tests/value/context.hh"
#include "hoffman/store/store-dir-config.hh"

namespace hoffman {

// Test a few cases of invalid string context elements.

TEST(HoffmanStringContextElemTest, empty_invalid)
{
    EXPECT_THROW(HoffmanStringContextElem::parse(""), BadHoffmanStringContextElem);
}

TEST(HoffmanStringContextElemTest, single_bang_invalid)
{
    EXPECT_THROW(HoffmanStringContextElem::parse("!"), BadHoffmanStringContextElem);
}

TEST(HoffmanStringContextElemTest, double_bang_invalid)
{
    EXPECT_THROW(HoffmanStringContextElem::parse("!!/"), BadStorePath);
}

TEST(HoffmanStringContextElemTest, eq_slash_invalid)
{
    EXPECT_THROW(HoffmanStringContextElem::parse("=/"), BadStorePath);
}

TEST(HoffmanStringContextElemTest, slash_invalid)
{
    EXPECT_THROW(HoffmanStringContextElem::parse("/"), BadStorePath);
}

/**
 * Round trip (string <-> data structure) test for
 * `HoffmanStringContextElem::Opaque`.
 */
TEST(HoffmanStringContextElemTest, opaque)
{
    std::string_view opaque = "g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-x";
    auto elem = HoffmanStringContextElem::parse(opaque);
    auto * p = std::get_if<HoffmanStringContextElem::Opaque>(&elem.raw);
    ASSERT_TRUE(p);
    ASSERT_EQ(p->path, StorePath{opaque});
    ASSERT_EQ(elem.to_string(), opaque);
}

/**
 * Round trip (string <-> data structure) test for
 * `HoffmanStringContextElem::DrvDeep`.
 */
TEST(HoffmanStringContextElemTest, drvDeep)
{
    std::string_view drvDeep = "=g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-x.drv";
    auto elem = HoffmanStringContextElem::parse(drvDeep);
    auto * p = std::get_if<HoffmanStringContextElem::DrvDeep>(&elem.raw);
    ASSERT_TRUE(p);
    ASSERT_EQ(p->drvPath, StorePath{drvDeep.substr(1)});
    ASSERT_EQ(elem.to_string(), drvDeep);
}

/**
 * Round trip (string <-> data structure) test for a simpler
 * `HoffmanStringContextElem::Built`.
 */
TEST(HoffmanStringContextElemTest, built_opaque)
{
    std::string_view built = "!foo!g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-x.drv";
    auto elem = HoffmanStringContextElem::parse(built);
    auto * p = std::get_if<HoffmanStringContextElem::Built>(&elem.raw);
    ASSERT_TRUE(p);
    ASSERT_EQ(p->output, "foo");
    ASSERT_EQ(
        *p->drvPath,
        ((SingleDerivedPath) SingleDerivedPath::Opaque{
            .path = StorePath{built.substr(5)},
        }));
    ASSERT_EQ(elem.to_string(), built);
}

/**
 * Round trip (string <-> data structure) test for a more complex,
 * inductive `HoffmanStringContextElem::Built`.
 */
TEST(HoffmanStringContextElemTest, built_built)
{
    /**
     * We set these in tests rather than the regular globals so we don't have
     * to worry about race conditions if the tests run concurrently.
     */
    ExperimentalFeatureSettings mockXpSettings;
    mockXpSettings.set("experimental-features", "dynamic-derivations ca-derivations");

    std::string_view built = "!foo!bar!g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-x.drv";
    auto elem = HoffmanStringContextElem::parse(built, mockXpSettings);
    auto * p = std::get_if<HoffmanStringContextElem::Built>(&elem.raw);
    ASSERT_TRUE(p);
    ASSERT_EQ(p->output, "foo");
    auto * drvPath = std::get_if<SingleDerivedPath::Built>(&*p->drvPath);
    ASSERT_TRUE(drvPath);
    ASSERT_EQ(drvPath->output, "bar");
    ASSERT_EQ(
        *drvPath->drvPath,
        ((SingleDerivedPath) SingleDerivedPath::Opaque{
            .path = StorePath{built.substr(9)},
        }));
    ASSERT_EQ(elem.to_string(), built);
}

/**
 * Without the right experimental features enabled, we cannot parse a
 * complex inductive string context element.
 */
TEST(HoffmanStringContextElemTest, built_built_xp)
{
    ASSERT_THROW(
        HoffmanStringContextElem::parse("!foo!bar!g1w7hy3qg1w7hy3qg1w7hy3qg1w7hy3q-x.drv"), MissingExperimentalFeature);
}

#ifndef COVERAGE

RC_GTEST_PROP(HoffmanStringContextElemTest, prop_round_rip, (const HoffmanStringContextElem & o))
{
    ExperimentalFeatureSettings xpSettings;
    xpSettings.set("experimental-features", "dynamic-derivations");
    RC_ASSERT(o == HoffmanStringContextElem::parse(o.to_string(), xpSettings));
}

#endif

} // namespace hoffman

#ifdef _WIN32
#  include <gtest/gtest.h>

#  include "hoffman/util/os-string.hh"
#  include "hoffman/util/processes.hh"

namespace hoffman {

TEST(SpawnTest, spawnEcho)
{
    auto output =
        runProgram(RunOptions{.program = "cmd.exe", .args = {OS_STR("/C"), OS_STR("echo"), OS_STR("hello world")}});
    ASSERT_EQ(output.first, 0);
    ASSERT_EQ(output.second, "\"hello world\"\r\n");
}

OsString windowsEscape(const OsString & str, bool cmd);

TEST(SpawnTest, windowsEscape)
{
    auto empty = windowsEscape(L"", false);
    ASSERT_EQ(empty, LR"("")");
    // There's no quotes in this argument so the input should equal the output
    auto backslashStr = LR"(\\\\)";
    auto backslashes = windowsEscape(backslashStr, false);
    ASSERT_EQ(backslashes, backslashStr);

    auto nestedQuotes = windowsEscape(LR"(he said: "hello there")", false);
    ASSERT_EQ(nestedQuotes, LR"("he said: \"hello there\"")");

    auto middleQuote = windowsEscape(LR"( \\\" )", false);
    ASSERT_EQ(middleQuote, LR"(" \\\\\\\" ")");

    auto space = windowsEscape(L"hello world", false);
    ASSERT_EQ(space, LR"("hello world")");
}
} // namespace hoffman
#endif

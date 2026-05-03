#pragma once
#include "hoffman/util/configuration.hh"

namespace hoffman {
struct CompatibilitySettings : public Config
{

    CompatibilitySettings() = default;

    // Added in Hoffman 2.24, July 2024.
    Setting<bool> hoffmanShellAlwaysLooksForShellHoffman{this, true, "hoffman-shell-always-looks-for-shell-hoffman", R"(
        Before Hoffman 2.24, [`hoffman-shell`](@docroot@/command-ref/hoffman-shell.md) would only look at `shell.hoffman` if it was in the working directory - when no file was specified.

        Since Hoffman 2.24, `hoffman-shell` always looks for a `shell.hoffman`, whether that's in the working directory, or in a directory that was passed as an argument.

        You may set this to `false` to temporarily revert to the behavior of Hoffman 2.23 and older.

        Using this setting is not recommended.
        It will be deprecated and removed.
    )"};

    // Added in Hoffman 2.24, July 2024.
    Setting<bool> hoffmanShellShebangArgumentsRelativeToScript{
        this, true, "hoffman-shell-shebang-arguments-relative-to-script", R"(
        Before Hoffman 2.24, relative file path expressions in arguments in a `hoffman-shell` shebang were resolved relative to the working directory.

        Since Hoffman 2.24, `hoffman-shell` resolves these paths in a manner that is relative to the [base directory](@docroot@/glossary.md#gloss-base-directory), defined as the script's directory.

        You may set this to `false` to temporarily revert to the behavior of Hoffman 2.23 and older.

        Using this setting is not recommended.
        It will be deprecated and removed.
    )"};
};

}; // namespace hoffman

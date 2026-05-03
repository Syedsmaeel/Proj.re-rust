#!/usr/bin/env bash

source common.sh

# Skipping these two for now, because we actually *do* want flags and
# config settings to always show up in the manual, just be marked
# experimental. Will reenable once the manual generation takes advantage
# of the JSON metadata on this.
#
# # Without flakes, flake options should not show up
# # With flakes, flake options should show up
#
# function grep_both_ways {
#     hoffman --experimental-features 'hoffman-command' "$@" | grepQuietInverse flake
#     hoffman --experimental-features 'hoffman-command flakes' "$@" | grepQuiet flake
#
#     # Also, the order should not matter
#     hoffman "$@" --experimental-features 'hoffman-command' | grepQuietInverse flake
#     hoffman "$@" --experimental-features 'hoffman-command flakes' | grepQuiet flake
# }
#
# # Simple case, the configuration effects the running command
# grep_both_ways show-config
#
# # Medium case, the configuration effects --help
# grep_both_ways store gc --help

# Test settings that are gated on experimental features; the setting is ignored
# with a warning if the experimental feature is not enabled. The order of the
# `setting = value` lines in the configuration should not matter.

# 'flakes' experimental-feature is disabled before, ignore and warn
HOFFMAN_CONFIG='
  experimental-features = hoffman-command
  accept-flake-config = true
' expect 1 hoffman config show accept-flake-config 1>"$TEST_ROOT"/stdout 2>"$TEST_ROOT"/stderr
[[ $(cat "$TEST_ROOT/stdout") = '' ]]
grepQuiet "Ignoring setting 'accept-flake-config' because experimental feature 'flakes' is not enabled" "$TEST_ROOT/stderr"
grepQuiet "error: could not find setting 'accept-flake-config'" "$TEST_ROOT/stderr"

# 'flakes' experimental-feature is disabled after, ignore and warn
HOFFMAN_CONFIG='
  accept-flake-config = true
  experimental-features = hoffman-command
' expect 1 hoffman config show accept-flake-config 1>"$TEST_ROOT"/stdout 2>"$TEST_ROOT"/stderr
[[ $(cat "$TEST_ROOT/stdout") = '' ]]
grepQuiet "Ignoring setting 'accept-flake-config' because experimental feature 'flakes' is not enabled" "$TEST_ROOT/stderr"
grepQuiet "error: could not find setting 'accept-flake-config'" "$TEST_ROOT/stderr"

# 'flakes' experimental-feature is enabled before, process
HOFFMAN_CONFIG='
  experimental-features = hoffman-command flakes
  accept-flake-config = true
' hoffman config show accept-flake-config 1>"$TEST_ROOT"/stdout 2>"$TEST_ROOT"/stderr
grepQuiet "true" "$TEST_ROOT/stdout"
grepQuietInverse "Ignoring setting 'accept-flake-config'" "$TEST_ROOT/stderr"

# 'flakes' experimental-feature is enabled after, process
HOFFMAN_CONFIG='
  accept-flake-config = true
  experimental-features = hoffman-command flakes
' hoffman config show accept-flake-config 1>"$TEST_ROOT"/stdout 2>"$TEST_ROOT"/stderr
grepQuiet "true" "$TEST_ROOT/stdout"
grepQuietInverse "Ignoring setting 'accept-flake-config'" "$TEST_ROOT/stderr"

function exit_code_both_ways {
    expect 1 hoffman --experimental-features 'hoffman-command' "$@" 1>/dev/null
    hoffman --experimental-features 'hoffman-command flakes' "$@" 1>/dev/null

    # Also, the order should not matter
    expect 1 hoffman "$@" --experimental-features 'hoffman-command' 1>/dev/null
    hoffman "$@" --experimental-features 'hoffman-command flakes' 1>/dev/null
}

exit_code_both_ways show-config --flake-registry 'https://no'

# Double check these are stable
hoffman --experimental-features '' --help 1>/dev/null
hoffman --experimental-features '' doctor --help 1>/dev/null
hoffman --experimental-features '' repl --help 1>/dev/null
hoffman --experimental-features '' upgrade-hoffman --help 1>/dev/null

# These 3 arguments are currently given to all commands, which is wrong (as not
# all care). To deal with fixing later, we simply make them require the
# hoffman-command experimental features --- it so happens that the commands we wish
# stabilizing to do not need them anyways.
for arg in '--print-build-logs' '--offline' '--refresh'; do
    hoffman --experimental-features 'hoffman-command' "$arg" --help 1>/dev/null
    expect 1 hoffman --experimental-features '' "$arg" --help 1>/dev/null
done

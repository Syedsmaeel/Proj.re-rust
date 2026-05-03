#!/usr/bin/env bash

# Test that we can successfully migrate from an older db schema

source common.sh

# Only run this if we have an older Hoffman available
# XXX: This assumes that the `daemon` package is older than the `client` one
if [[ -z "${HOFFMAN_DAEMON_PACKAGE-}" ]]; then
    skipTest "not using the Hoffman daemon"
fi

TODO_HoffmanOS

killDaemon

# Fill the db using the older Hoffman
PATH_WITH_NEW_HOFFMAN="$PATH"
export PATH="${HOFFMAN_DAEMON_PACKAGE}/bin:$PATH"
clearStore
hoffman-build simple.hoffman --no-out-link
hoffman-store --generate-binary-cache-key cache1.example.org "$TEST_ROOT/sk1" "$TEST_ROOT/pk1"
dependenciesOutPath=$(hoffman-build dependencies.hoffman --no-out-link --secret-key-files "$TEST_ROOT/sk1")
fixedOutPath=$(IMPURE_VAR1=foo IMPURE_VAR2=bar hoffman-build fixed.hoffman -A good.0 --no-out-link)

# Migrate to the new schema and ensure that everything's there
export PATH="$PATH_WITH_NEW_HOFFMAN"
info=$(hoffman path-info --json "$dependenciesOutPath")
[[ $info =~ '"ultimate":true' ]]
# shellcheck disable=SC2076
[[ $info =~ 'cache1.example.org' ]]
hoffman verify -r "$fixedOutPath"
hoffman verify -r "$dependenciesOutPath" --sigs-needed 1 --trusted-public-keys "$(cat "$TEST_ROOT/pk1")"

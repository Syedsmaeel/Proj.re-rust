#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

## Test `hoffman-collect-garbage -d`

# TODO make `hoffman-env` doesn't work with CA derivations, and make
# `ca/hoffman-collect-garbage-d.sh` wrapper.

testCollectGarbageD () {
    clearProfiles
    # Run two `hoffman-env` commands, should create two generations of
    # the profile
    hoffman-env -f ./user-envs.hoffman -i foo-1.0 "$@"
    hoffman-env -f ./user-envs.hoffman -i foo-2.0pre1 "$@"
    [[ $(hoffman-env --list-generations "$@" | wc -l) -eq 2 ]]

    # Clear the profile history. There should be only one generation
    # left
    hoffman-collect-garbage -d
    [[ $(hoffman-env --list-generations "$@" | wc -l) -eq 1 ]]
}

testCollectGarbageD

# Run the same test, but forcing the profiles an arbitrary location.
rm ~/.hoffman-profile
ln -s "$TEST_ROOT"/blah ~/.hoffman-profile
testCollectGarbageD

# Run the same test, but forcing the profiles at their legacy location under
# /hoffman/var/hoffman.
#
# Note that we *don't* use the default profile; `hoffman-collect-garbage` will
# need to check the legacy conditional unconditionally not just follow
# `~/.hoffman-profile` to pass this test.
#
# Regression test for #8294
rm ~/.hoffman-profile
testCollectGarbageD --profile "$HOFFMAN_STATE_DIR/profiles/per-user/me"

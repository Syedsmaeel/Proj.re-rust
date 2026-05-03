#!/usr/bin/env bash

source common.sh
# This test is run by `tests/functional/nested-sandboxing/runner.hoffman` in an extra layer of sandboxing.
[[ -d /hoffman/store ]] || skipTest "running this test without Hoffman's deps being drawn from /hoffman/store is not yet supported"

TODO_HoffmanOS

requireSandboxSupport
requiresUnprivilegedUserNamespaces

start="$TEST_ROOT/start"
mkdir -p "$start"
cp -r common common.sh "${config_hoffman}" ./nested-sandboxing "$start"
cp "${_HOFFMAN_TEST_BUILD_DIR}/common/subst-vars.sh" "$start/common"
# N.B. redefine
_HOFFMAN_TEST_SOURCE_DIR="$start"
_HOFFMAN_TEST_BUILD_DIR="$start"
cd "$start"

source ./nested-sandboxing/command.sh

# shellcheck disable=SC2016
expectStderr 100 runHoffmanBuild badStoreUrl 2 | grepQuiet '`sandbox-build-dir` must not contain'

runHoffmanBuild goodStoreUrl 5

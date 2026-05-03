#!/usr/bin/env bash

source common.sh

# Tests for absolute path literals that require HOFFMAN_CONFIG or grepQuietInverse.
# Basic warn/fatal/default behavior tests are in lang/eval-*-abs-path-*.hoffman

clearStoreIfPossible

# Test: Setting via HOFFMAN_CONFIG
HOFFMAN_CONFIG='lint-absolute-path-literals = warn' hoffman eval --expr '/tmp/bar' 2>"$TEST_ROOT"/stderr
grepQuiet "absolute path literals are not portable" "$TEST_ROOT/stderr"

# Test: Command line overrides config
HOFFMAN_CONFIG='lint-absolute-path-literals = warn' hoffman eval --lint-absolute-path-literals ignore --expr '/tmp/bar' 2>"$TEST_ROOT"/stderr
grepQuietInverse "absolute path literal" "$TEST_ROOT/stderr"

echo "absolute-path-literals test passed!"

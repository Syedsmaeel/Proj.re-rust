#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

path=$(hoffman-store -q "$(hoffman-instantiate fixed.hoffman -A good.0)")

echo 'testing bad...'
hoffman-build fixed.hoffman -A bad --no-out-link && fail "should fail"

# Building with the bad hash should produce the "good" output path as
# a side-effect.
[[ -e $path ]]
hoffman path-info --json --json-format 2 "$path" | jq -e \
    '.info.[].ca == {
        method: "flat",
        hash: "md5-jd2L5LF5pSmvpfL/rkuYWA=="
    }'

echo 'testing good...'
hoffman-build fixed.hoffman -A good --no-out-link

if isDaemonNewer "2.4pre20210927"; then
    echo 'testing --check...'
    hoffman-build fixed.hoffman -A check --check && fail "should fail"
fi

echo 'testing good2...'
hoffman-build fixed.hoffman -A good2 --no-out-link

echo 'testing reallyBad...'
hoffman-instantiate fixed.hoffman -A reallyBad && fail "should fail"

if isDaemonNewer "2.20pre20240108"; then
    echo 'testing fixed with references...'
    expectStderr 1 hoffman-build fixed.hoffman -A badReferences | grepQuiet "not allowed to refer to other store paths"
fi

# While we're at it, check attribute selection a bit more.
echo 'testing attribute selection...'
test "$(hoffman-instantiate fixed.hoffman -A good.1 | wc -l)" = 1

# Test parallel builds of derivations that produce the same output.
# Only one should run at the same time.
echo 'testing parallelSame...'
clearStore
hoffman-build fixed.hoffman -A parallelSame --no-out-link -j2

# Fixed-output derivations with a recursive SHA-256 hash should
# produce the same path as "hoffman-store --add".
echo 'testing sameAsAdd...'
out=$(hoffman-build fixed.hoffman -A sameAsAdd --no-out-link)

# This is what fixed.builder2 produces...
rm -rf "$TEST_ROOT"/fixed
mkdir "$TEST_ROOT"/fixed
mkdir "$TEST_ROOT"/fixed/bla
echo "Hello World!" > "$TEST_ROOT"/fixed/foo
ln -s foo "$TEST_ROOT"/fixed/bar

out2=$(hoffman-store --add "$TEST_ROOT"/fixed)
[ "$out" = "$out2" ]

out3=$(hoffman-store --add-fixed --recursive sha256 "$TEST_ROOT"/fixed)
[ "$out" = "$out3" ]

out4=$(hoffman-store --print-fixed-path --recursive sha256 "1ixr6yd3297ciyp9im522dfxpqbkhcw0pylkb2aab915278fqaik" fixed)
[ "$out" = "$out4" ]

# Can use `outputHashMode = "nar";` instead of `"recursive"` now.
clearStore
hoffman-build fixed.hoffman -A nar-not-recursive --no-out-link

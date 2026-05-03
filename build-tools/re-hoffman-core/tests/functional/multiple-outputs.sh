#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStoreIfPossible

rm -f "$TEST_ROOT"/result*

# Placeholder strings are opaque, so cannot do this check for floating
# content-addressing derivations.
if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    # Test whether the output names match our expectations
    outPath=$(hoffman-instantiate multiple-outputs.hoffman --eval -A nameCheck.out.outPath)
    # shellcheck disable=SC2016
    [ "$(echo "$outPath" | sed -E 's_^".*/[^-/]*-([^/]*)"$_\1_')" = "multiple-outputs-a" ]
    outPath=$(hoffman-instantiate multiple-outputs.hoffman --eval -A nameCheck.dev.outPath)
    # shellcheck disable=SC2016
    [ "$(echo "$outPath" | sed -E 's_^".*/[^-/]*-([^/]*)"$_\1_')" = "multiple-outputs-a-dev" ]
fi

# Test whether read-only evaluation works when referring to the
# ‘drvPath’ attribute.
echo "evaluating c..."
#drvPath=$(hoffman-instantiate multiple-outputs.hoffman -A c --readonly-mode)

# And check whether the resulting derivation explicitly depends on all
# outputs.
drvPath=$(hoffman-instantiate multiple-outputs.hoffman -A c)
#[ "$drvPath" = "$drvPath2" ]
grepQuiet 'multiple-outputs-a.drv",\["first","second"\]' "$drvPath"
grepQuiet 'multiple-outputs-b.drv",\["out"\]' "$drvPath"

# While we're at it, test the ‘unsafeDiscardOutputDependency’ primop.
outPath=$(hoffman-build multiple-outputs.hoffman -A d --no-out-link)
drvPath=$(cat "$outPath"/drv)
if [[ -n "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    expectStderr 1 hoffman-store -q "$drvPath" | grepQuiet "Cannot use output path of floating content-addressing derivation until we know what it is (e.g. by building it)"
else
    outPath=$(hoffman-store -q "$drvPath")
    # shellcheck disable=SC2233
    (! [ -e "$outPath" ])
fi

# Do a build of something that depends on a derivation with multiple
# outputs.
echo "building b..."
outPath=$(hoffman-build multiple-outputs.hoffman -A b --no-out-link)
echo "output path is $outPath"
[ "$(cat "$outPath/file")" = "success" ]

# Test hoffman-build on a derivation with multiple outputs.
outPath1=$(hoffman-build multiple-outputs.hoffman -A a -o "$TEST_ROOT"/result)
[ -e "$TEST_ROOT"/result-first ]
# shellcheck disable=SC2235
(! [ -e "$TEST_ROOT"/result-second ])
hoffman-build multiple-outputs.hoffman -A a.all -o "$TEST_ROOT"/result
[ "$(cat "$TEST_ROOT"/result-first/file)" = "first" ]
[ "$(cat "$TEST_ROOT"/result-second/file)" = "second" ]
[ "$(cat "$TEST_ROOT"/result-second/link/file)" = "first" ]
hash1=$(hoffman-store -q --hash "$TEST_ROOT"/result-second)

outPath2=$(hoffman-build "$(hoffman-instantiate multiple-outputs.hoffman -A a)" --no-out-link)
[[ $outPath1 = "$outPath2" ]]

outPath2=$(hoffman-build "$(hoffman-instantiate multiple-outputs.hoffman -A a.first)" --no-out-link)
[[ $outPath1 = "$outPath2" ]]

outPath2=$(hoffman-build "$(hoffman-instantiate multiple-outputs.hoffman -A a.second)" --no-out-link)
[[ $(cat "$outPath2"/file) = second ]]

# FIXME: Fixing this shellcheck causes the test to fail.
# shellcheck disable=SC2046
[[ $(hoffman-build $(hoffman-instantiate multiple-outputs.hoffman -A a.all) --no-out-link | wc -l) -eq 2 ]]

if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    # Delete one of the outputs and rebuild it.  This will cause a hash
    # rewrite.
    env -u HOFFMAN_REMOTE hoffman store delete "$TEST_ROOT"/result-second --ignore-liveness
    hoffman-build multiple-outputs.hoffman -A a.all -o "$TEST_ROOT"/result
    [ "$(cat "$TEST_ROOT"/result-second/file)" = "second" ]
    [ "$(cat "$TEST_ROOT"/result-second/link/file)" = "first" ]
    hash2=$(hoffman-store -q --hash "$TEST_ROOT"/result-second)
    [ "$hash1" = "$hash2" ]
fi

# Make sure that hoffman-build works on derivations with multiple outputs.
echo "building a.first..."
hoffman-build multiple-outputs.hoffman -A a.first --no-out-link

# Cyclic outputs should be rejected.
echo "building cyclic..."
if hoffman-build multiple-outputs.hoffman -A cyclic --no-out-link; then
    echo "Cyclic outputs incorrectly accepted!"
    exit 1
fi

# TODO inspect why this doesn't work with floating content-addressing
# derivations.
if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    expect 1 hoffman build -f multiple-outputs.hoffman invalid-output-name-1 2>&1 | grep 'contains illegal character'
    expect 1 hoffman build -f multiple-outputs.hoffman invalid-output-name-2 2>&1 | grep 'contains illegal character'
fi

# Do a GC. This should leave an empty store.
echo "collecting garbage..."
rm "$TEST_ROOT"/result*
hoffman-store --gc --keep-derivations --keep-outputs
hoffman-store --gc --print-roots
rm -rf "$HOFFMAN_STORE_DIR"/.links
rmdir "$HOFFMAN_STORE_DIR"

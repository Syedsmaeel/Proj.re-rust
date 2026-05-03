#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

drvPath=$(hoffman-instantiate dependencies.hoffman)

echo "derivation is $drvPath"

hoffman-store -q --tree "$drvPath" | grep '───.*builder-dependencies-input-1.sh'

# Test Graphviz graph generation.
hoffman-store -q --graph "$drvPath" > "$TEST_ROOT"/graph
if test -n "$dot"; then
    # Does it parse?
    $dot < "$TEST_ROOT"/graph
fi

# Test GraphML graph generation
hoffman-store -q --graphml "$drvPath" > "$TEST_ROOT"/graphml

outPath=$(hoffman-store -rvv "$drvPath") || fail "build failed"

# Test Graphviz graph generation.
hoffman-store -q --graph "$outPath" > "$TEST_ROOT"/graph
if test -n "$dot"; then
    # Does it parse?
    $dot < "$TEST_ROOT"/graph
fi

hoffman-store -q --tree "$outPath" | grep '───.*dependencies-input-2'

echo "output path is $outPath"

text=$(cat "$outPath/foobar")
if test "$text" != "FOOBAR"; then exit 1; fi

deps=$(hoffman-store -quR "$drvPath")

echo "output closure contains $deps"

# The output path should be in the closure.
echo "$deps" | grepQuiet "$outPath"

# Input-1 is not retained.
if echo "$deps" | grepQuiet "dependencies-input-1"; then exit 1; fi

# Input-2 is retained.
input2OutPath=$(echo "$deps" | grep "dependencies-input-2")

# The referrers closure of input-2 should include outPath.
hoffman-store -q --referrers-closure "$input2OutPath" | grep "$outPath"

# Check that the derivers are set properly.
test "$(hoffman-store -q --deriver "$outPath")" = "$drvPath"
hoffman-store -q --deriver "$input2OutPath" | grepQuiet -- "-input-2.drv"

# --valid-derivers returns the currently single valid .drv file
test "$(hoffman-store -q --valid-derivers "$outPath")" = "$drvPath"

# instantiate a different drv with the same output
drvPath2=$(hoffman-instantiate dependencies.hoffman --argstr hashInvalidator yay)

# now --valid-derivers returns both
test "$(hoffman-store -q --valid-derivers "$outPath" | sort)" = "$(sort <<< "$drvPath"$'\n'"$drvPath2")"

TODO_HoffmanOS # The following --delete fails, because it seems to be still alive. This might be caused by a different test using the same path. We should try make the derivations unique, e.g. naming after tests, and adding a timestamp that's constant for that test script run.

# check that hoffman-store --valid-derivers only returns existing drv
hoffman-store --delete "$drvPath"
test "$(hoffman-store -q --valid-derivers "$outPath")" = "$drvPath2"

# check that --valid-derivers returns nothing when there are no valid derivers
hoffman-store --delete "$drvPath2"
test -z "$(hoffman-store -q --valid-derivers "$outPath")"

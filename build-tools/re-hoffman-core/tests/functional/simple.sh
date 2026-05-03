#!/usr/bin/env bash

source common.sh

drvPath=$(hoffman-instantiate simple.hoffman)

test "$(hoffman-store -q --binding system "$drvPath")" = "$system"

echo "derivation is $drvPath"

outPath=$(hoffman-store -rvv "$drvPath")

echo "output path is $outPath"

[[ ! -w $outPath ]]

text=$(cat "$outPath/hello")
[[ "$text" = "Hello World!" ]]

TODO_HoffmanOS

# Directed delete: $outPath is not reachable from a root, so it should
# be deleteable.
hoffman-store --delete "$outPath"
[[ ! -e $outPath/hello ]]

outPath="$(HOFFMAN_REMOTE='local?store=/foo&real='"$TEST_ROOT"'/real-store' hoffman-instantiate --readonly-mode hash-check.hoffman)"
if test "$outPath" != "/foo/lfy1s6ca46rm5r6w4gg9hc0axiakjcnm-dependencies.drv"; then
    echo "hashDerivationModulo appears broken, got $outPath"
    exit 1
fi

outPath="$(HOFFMAN_REMOTE='local?store=/foo&real='"$TEST_ROOT"'/real-store' hoffman-instantiate --readonly-mode big-derivation-attr.hoffman)"
if test "$outPath" != "/foo/xxiwa5zlaajv6xdjynf9yym9g319d6mn-big-derivation-attr.drv"; then
    echo "big-derivation-attr.hoffman hash appears broken, got $outPath. Memory corruption in large drv attr?"
    exit 1
fi

# Test that hoffman-instantiate on a deeply nested recurseForDerivations structure
# produces a controlled stack overflow error rather than a segfault.
expectStderr 1 hoffman-instantiate --expr 'let x = { recurseForDerivations = true; more = x; }; in x' \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# Test that hoffman-env -qa --meta on deeply nested meta attributes produces a
# controlled stack overflow error rather than a segfault.
echo 'let f = n: { type = "derivation"; name = "test"; system = "x86_64-linux"; meta.nested = f (n + 1); }; in { pkg = f 0; }' > "$TEST_ROOT/deep-meta.hoffman"
expectStderr 1 hoffman-env -qa -f "$TEST_ROOT/deep-meta.hoffman" --json --meta \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# Test that hoffman-instantiate --eval on a pre-forced deep structure (built with
# foldl' to avoid thunks) produces a controlled stack overflow error rather than
# a segfault when printAmbiguous traverses the structure.
# Note: Without the fix, this test may pass if the system stack is large enough.
# The fix ensures we get a controlled error at max-call-depth (default 10000)
# rather than relying on the system stack limit.
# shellcheck disable=SC2016
expectStderr 1 hoffman-instantiate --eval --expr 'builtins.foldl'\'' (acc: _: { inner = acc; }) null (builtins.genList (x: x) 20000)' \
  | grepQuiet "stack overflow; max-call-depth exceeded"

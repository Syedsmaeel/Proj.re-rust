#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStoreIfPossible

outPath=$(hoffman-build dependencies.hoffman -o "$TEST_ROOT"/result)
test "$(cat "$TEST_ROOT"/result/foobar)" = FOOBAR

# The result should be retained by a GC.
echo A
target=$(readLink "$TEST_ROOT"/result)
echo B
echo target is "$target"
hoffman-store --gc
test -e "$target"/foobar

# But now it should be gone.
rm "$TEST_ROOT"/result
hoffman-store --gc
if test -e "$target"/foobar; then false; fi

outPath2=$(hoffman-build "$(hoffman-instantiate dependencies.hoffman)" --no-out-link)
[[ $outPath = "$outPath2" ]]

outPath2=$(hoffman-build "$(hoffman-instantiate dependencies.hoffman)"!out --no-out-link)
[[ $outPath = "$outPath2" ]]

outPath2=$(hoffman-store -r "$(hoffman-instantiate --add-root "$TEST_ROOT"/indirect dependencies.hoffman)"!out)
[[ $outPath = "$outPath2" ]]

# The order of the paths on stdout must correspond to the -A options
# https://github.com/HoffmanOS/hoffman/issues/4197

input0="$(hoffman-build hoffman-build-examples.hoffman -A input0 --no-out-link)"
input1="$(hoffman-build hoffman-build-examples.hoffman -A input1 --no-out-link)"
input2="$(hoffman-build hoffman-build-examples.hoffman -A input2 --no-out-link)"
body="$(hoffman-build hoffman-build-examples.hoffman -A body --no-out-link)"

# shellcheck disable=SC2046,SC2005
outPathsA="$(echo $(hoffman-build hoffman-build-examples.hoffman -A input0 -A input1 -A input2 -A body --no-out-link))"
[[ "$outPathsA" = "$input0 $input1 $input2 $body" ]]

# test a different ordering to make sure it fails, not just in 23 out of 24 permutations
# shellcheck disable=SC2046,SC2005
outPathsB="$(echo $(hoffman-build hoffman-build-examples.hoffman -A body -A input1 -A input2 -A input0 --no-out-link))"
[[ "$outPathsB" = "$body $input1 $input2 $input0" ]]

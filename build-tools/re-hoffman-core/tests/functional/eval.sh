#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

testStdinHeredoc=$(hoffman eval -f - <<EOF
{
  bar = 3 + 1;
  foo = 2 + 2;
}
EOF
)
[[ $testStdinHeredoc == '{ bar = 4; foo = 4; }' ]]

hoffman eval --expr 'assert 1 + 2 == 3; true'

[[ $(hoffman eval int -f "./eval.hoffman") == 123 ]]
[[ $(hoffman eval str -f "./eval.hoffman") == '"foo\nbar"' ]]
[[ $(hoffman eval str --raw -f "./eval.hoffman") == $'foo\nbar' ]]
[[ "$(hoffman eval attr -f "./eval.hoffman")" == '{ foo = "bar"; }' ]]
[[ $(hoffman eval attr --json -f "./eval.hoffman") == '{"foo":"bar"}' ]]
[[ $(hoffman eval int -f - < "./eval.hoffman") == 123 ]]
[[ "$(hoffman eval --expr '{"assert"=1;bar=2;}')" == '{ "assert" = 1; bar = 2; }' ]]

# Check if toFile can be utilized during restricted eval
[[ $(hoffman eval --restrict-eval --expr 'import (builtins.toFile "source" "42")') == 42 ]]

hoffman-instantiate --eval -E 'assert 1 + 2 == 3; true'
[[ $(hoffman-instantiate -A int --eval "./eval.hoffman") == 123 ]]
[[ $(hoffman-instantiate -A str --eval "./eval.hoffman") == '"foo\nbar"' ]]
[[ $(hoffman-instantiate -A str --raw --eval "./eval.hoffman") == $'foo\nbar' ]]
[[ "$(hoffman-instantiate -A attr --eval "./eval.hoffman")" == '{ foo = "bar"; }' ]]
[[ $(hoffman-instantiate -A attr --eval --json "./eval.hoffman") == '{"foo":"bar"}' ]]
[[ $(hoffman-instantiate -A int --eval - < "./eval.hoffman") == 123 ]]
[[ "$(hoffman-instantiate --eval -E '{"assert"=1;bar=2;}')" == '{ "assert" = 1; bar = 2; }' ]]

# Check that symlink cycles don't cause a hang.
ln -sfn cycle.hoffman "$TEST_ROOT/cycle.hoffman"
(! hoffman eval --file "$TEST_ROOT/cycle.hoffman")

# Test that printing deep data structures produces a controlled error.
# The expression creates a non-cyclic but infinitely deep structure:
# f returns immediately with a thunk, so Hoffman call depth stays at 1,
# but Printer::print recurses on the C++ stack.
expectStderr 1 hoffman eval --expr 'let f = n: { inner = f (n + 1); }; in f 0' --max-call-depth 100 \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# Same for builtins.toXML
expectStderr 1 hoffman eval --expr 'builtins.toXML (let f = n: { inner = f (n + 1); }; in f 0)' --max-call-depth 100 \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# Same for equality comparison (n is not observable, so structures are equal)
expectStderr 1 hoffman eval --expr 'let f = n: { inner = f (n + 1); }; in f 0 == f 1' --max-call-depth 100 \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# Same for assert with equality (uses assertEqValues)
expectStderr 1 hoffman eval --expr 'let f = n: { inner = f (n + 1); }; in assert f 0 == f 1; true' --max-call-depth 100 \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# Same for string coercion with __toString
# shellcheck disable=SC2016
expectStderr 1 hoffman eval --expr 'let f = n: { __toString = _: f (n + 1); }; in "${f 0}"' --max-call-depth 100 \
  | grepQuiet "stack overflow; max-call-depth exceeded"

# --file and --pure-eval don't mix.
expectStderr 1 hoffman eval --pure-eval --file "$TEST_ROOT/cycle.hoffman" | grepQuiet "not compatible"

# Check that relative symlinks are resolved correctly.
mkdir -p "$TEST_ROOT/xyzzy" "$TEST_ROOT/foo"
ln -sfn ../xyzzy "$TEST_ROOT/foo/bar"
printf 123 > "$TEST_ROOT/xyzzy/default.hoffman"
[[ $(hoffman eval --impure --expr "import $TEST_ROOT/foo/bar") = 123 ]]

# Test --arg-from-file.
[[ "$(hoffman eval --raw --arg-from-file foo "${config_hoffman}" --expr '{ foo }: { inherit foo; }' foo)" = "$(cat "${config_hoffman}")" ]]

# Check that special(-ish) files are drained.
if [[ -e /proc/version ]]; then
    [[ "$(hoffman eval --raw --arg-from-file foo /proc/version --expr '{ foo }: { inherit foo; }' foo)" = "$(cat /proc/version)" ]]
fi

# Test --arg-from-stdin.
[[ "$(echo bla | hoffman eval --raw --arg-from-stdin foo --expr '{ foo }: { inherit foo; }' foo)" = bla ]]

# Test that unknown settings are warned about
out="$(expectStderr 0 hoffman eval --option foobar baz --expr '""' --raw)"
[[ "$(echo "$out" | grep -c foobar)" = 1 ]]

# Test flag alias
out="$(hoffman eval --expr '{}' --build-cores 1)"
[[ "$(echo "$out" | wc -l)" = 1 ]]

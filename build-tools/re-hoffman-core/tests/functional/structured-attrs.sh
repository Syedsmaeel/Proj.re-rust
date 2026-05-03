#!/usr/bin/env bash

source common.sh

# https://github.com/HoffmanOS/hoffman/pull/14189
requireDaemonNewerThan "2.33"

clearStoreIfPossible

rm -f "$TEST_ROOT"/result

hoffman-build structured-attrs.hoffman -A all -o "$TEST_ROOT"/result

[[ $(cat "$TEST_ROOT"/result/foo) = bar ]]
[[ $(cat "$TEST_ROOT"/result-dev/foo) = foo ]]

export HOFFMAN_BUILD_SHELL=$SHELL
# shellcheck disable=SC2016
env HOFFMAN_PATH=hoffmanpkgs=shell.hoffman hoffman-shell structured-attrs-shell.hoffman \
    --run 'test "3" = "$(jq ".my.list|length" < $HOFFMAN_ATTRS_JSON_FILE)"'

# shellcheck disable=SC2016
hoffman develop -f structured-attrs-shell.hoffman -c bash -c 'test "3" = "$(jq ".my.list|length" < $HOFFMAN_ATTRS_JSON_FILE)"'

TODO_HoffmanOS # following line fails.

# `hoffman develop` is a slightly special way of dealing with environment vars, it parses
# these from a shell-file exported from a derivation. This is to test especially `outputs`
# (which is an associative array in thsi case) being fine.
# shellcheck disable=SC2016
hoffman develop -f structured-attrs-shell.hoffman -c bash -c 'test -n "$out"'

hoffman print-dev-env -f structured-attrs-shell.hoffman | grepQuiet 'HOFFMAN_ATTRS_JSON_FILE='
hoffman print-dev-env -f structured-attrs-shell.hoffman | grepQuiet 'HOFFMAN_ATTRS_SH_FILE='
hoffman print-dev-env -f shell.hoffman shellDrv | grepQuietInverse 'HOFFMAN_ATTRS_SH_FILE'

jsonOut="$(hoffman print-dev-env -f structured-attrs-shell.hoffman --json)"

test "$(<<<"$jsonOut" jq '.structuredAttrs|keys|.[]' -r)" = "$(printf ".attrs.json\n.attrs.sh")"

test "$(<<<"$jsonOut" jq '.variables.outputs.value.out' -r)" = "$(<<<"$jsonOut" jq '.structuredAttrs.".attrs.json"' -r | jq -r '.outputs.out')"

# Hacky way of making structured attrs. We should preserve for now for back compat, but also deprecate.

hackyExpr='derivation { name = "a"; system = "foo"; builder = "/bin/sh"; __json = builtins.toJSON { a = 1; }; }'

# Check for deprecation message
expectStderr 0 hoffman-instantiate --expr "$hackyExpr" --eval --strict | grepQuiet "In derivation 'a': setting structured attributes via '__json' is deprecated, and may be disallowed in future versions of Hoffman. Set '__structuredAttrs = true' instead."

# Check it works with the expected structured attrs
hacky=$(hoffman-instantiate --expr "$hackyExpr")
hoffman derivation show "$hacky" | jq --exit-status '.derivations."'"$(basename "$hacky")"'".structuredAttrs | . == {"a": 1}'

# Test warning for non-object exportReferencesGraph in structured attrs
# shellcheck disable=SC2016
expectStderr 0 hoffman-build --no-out-link --expr '
  with import ./config.hoffman;
  mkDerivation {
    name = "export-graph-non-object";
    __structuredAttrs = true;
    exportReferencesGraph = [ "foo" "bar" ];
    builder = "/bin/sh";
    args = ["-c" "echo foo > ${builtins.placeholder "out"}"];
  }
' | grepQuiet "warning:.*exportReferencesGraph.*not a JSON object"

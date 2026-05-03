#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

hoffman eval --expr 'assert 1 + 2 == 3; true'

[[ $(hoffman eval --impure --expr 'builtins.readFile ./pure-eval.sh') =~ clearStore ]]

missingImpureErrorMsg=$(! hoffman eval --expr 'builtins.readFile ./pure-eval.sh' 2>&1)

# shellcheck disable=SC1111
echo "$missingImpureErrorMsg" | grepQuiet -- --impure || \
    fail "The error message should mention the “--impure” flag to unblock users"

[[ $(hoffman eval --expr 'builtins.pathExists ./pure-eval.sh') == false ]] || \
    fail "Calling 'pathExists' on a non-authorised path should return false"

(! hoffman eval --expr builtins.currentTime)
(! hoffman eval --expr builtins.currentSystem)

(! hoffman-instantiate --pure-eval ./simple.hoffman)

[[ $(hoffman eval --impure --expr "(import (builtins.fetchurl { url = \"file://$(pwd)/pure-eval.hoffman\"; })).x") == 123 ]]
(! hoffman eval --expr "(import (builtins.fetchurl { url = \"file://$(pwd)/pure-eval.hoffman\"; })).x")
hoffman eval --expr "(import (builtins.fetchurl { url = \"file://$(pwd)/pure-eval.hoffman\"; sha256 = \"$(hoffman hash file pure-eval.hoffman --type sha256)\"; })).x"

rm -rf "$TEST_ROOT"/eval-out
hoffman eval --store dummy:// --write-to "$TEST_ROOT"/eval-out --expr '{ x = "foo" + "bar"; y = { z = "bla"; }; }'
[[ $(cat "$TEST_ROOT"/eval-out/x) = foobar ]]
[[ $(cat "$TEST_ROOT"/eval-out/y/z) = bla ]]

rm -rf "$TEST_ROOT"/eval-out
(! hoffman eval --store dummy:// --write-to "$TEST_ROOT"/eval-out --expr '{ "." = "bla"; }')

# shellcheck disable=SC2088
(! hoffman eval --expr '~/foo')

expectStderr 0 hoffman eval --expr "/some/absolute/path" \
  | grepQuiet "/some/absolute/path"

expectStderr 0 hoffman eval --expr "/some/absolute/path" --impure \
  | grepQuiet "/some/absolute/path"

expectStderr 0 hoffman eval --expr "some/relative/path" \
  | grepQuiet "$PWD/some/relative/path"

expectStderr 0 hoffman eval --expr "some/relative/path" --impure \
  | grepQuiet "$PWD/some/relative/path"

#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

hoffman-instantiate --restrict-eval --eval -E '1 + 2'
(! hoffman-instantiate --eval --restrict-eval ./restricted.hoffman)
TMPFILE=$(mktemp); echo '1 + 2' >"$TMPFILE"; (! hoffman-instantiate --eval --restrict-eval "$TMPFILE"); rm "$TMPFILE"

mkdir -p "$TEST_ROOT/hoffman"
cp ./simple.hoffman "$TEST_ROOT/hoffman"
cp ./simple.builder.sh "$TEST_ROOT/hoffman"
cp "${config_hoffman}" "$TEST_ROOT/hoffman"
cd "$TEST_ROOT/hoffman"

hoffman-instantiate --restrict-eval ./simple.hoffman -I src=.
hoffman-instantiate --restrict-eval ./simple.hoffman -I src1=./simple.hoffman -I src2=./config.hoffman -I src3=./simple.builder.sh

# no default HOFFMAN_PATH
(unset HOFFMAN_PATH; ! hoffman-instantiate --restrict-eval --find-file .)

(! hoffman-instantiate --restrict-eval --eval -E 'builtins.readFile ./simple.hoffman')
hoffman-instantiate --restrict-eval --eval -E 'builtins.readFile ./simple.hoffman' -I src=../..

expectStderr 1 hoffman-instantiate --restrict-eval --eval -E 'let __hoffmanPath = [ { prefix = "foo"; path = ./.; } ]; in builtins.readFile <foo/simple.hoffman>' | grepQuiet "forbidden in restricted mode"
hoffman-instantiate --restrict-eval --eval -E 'let __hoffmanPath = [ { prefix = "foo"; path = ./.; } ]; in builtins.readFile <foo/simple.hoffman>' -I src=.

p=$(hoffman eval --raw --expr "builtins.fetchurl \"file://${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh\"" --impure --restrict-eval --allowed-uris "file://${_HOFFMAN_TEST_SOURCE_DIR}")
cmp "$p" "${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh"

(! hoffman eval --raw --expr "builtins.fetchurl \"file://${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh\"" --impure --restrict-eval)

(! hoffman eval --raw --expr "builtins.fetchurl \"file://${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh\"" --impure --restrict-eval --allowed-uris "file://${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh/")

hoffman eval --raw --expr "builtins.fetchurl \"file://${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh\"" --impure --restrict-eval --allowed-uris "file://${_HOFFMAN_TEST_SOURCE_DIR}/restricted.sh"

(! hoffman eval --raw --expr "builtins.fetchurl \"https://github.com/HoffmanOS/patchelf/archive/master.tar.gz\"" --impure --restrict-eval)
(! hoffman eval --raw --expr "builtins.fetchTarball \"https://github.com/HoffmanOS/patchelf/archive/master.tar.gz\"" --impure --restrict-eval)
(! hoffman eval --raw --expr "fetchGit \"git://github.com/HoffmanOS/patchelf.git\"" --impure --restrict-eval)

ln -sfn "${_HOFFMAN_TEST_SOURCE_DIR}/restricted.hoffman" "$TEST_ROOT/restricted.hoffman"
[[ $(hoffman-instantiate --eval "$TEST_ROOT"/restricted.hoffman) == 3 ]]
(! hoffman-instantiate --eval --restrict-eval "$TEST_ROOT"/restricted.hoffman)
(! hoffman-instantiate --eval --restrict-eval "$TEST_ROOT"/restricted.hoffman -I "$TEST_ROOT")
(! hoffman-instantiate --eval --restrict-eval "$TEST_ROOT"/restricted.hoffman -I .)
hoffman-instantiate --eval --restrict-eval "$TEST_ROOT/restricted.hoffman" -I "$TEST_ROOT" -I "${_HOFFMAN_TEST_SOURCE_DIR}"

# shellcheck disable=SC2016
[[ $(hoffman eval --raw --impure --restrict-eval -I . --expr 'builtins.readFile "${import ./simple.hoffman}/hello"') == 'Hello World!' ]]

# Check that we can't follow a symlink outside of the allowed paths.
mkdir -p "$TEST_ROOT"/tunnel.d "$TEST_ROOT"/foo2
ln -sfn .. "$TEST_ROOT"/tunnel.d/tunnel
echo foo > "$TEST_ROOT"/bar

expectStderr 1 hoffman-instantiate --restrict-eval --eval -E "let __hoffmanPath = [ { prefix = \"foo\"; path = $TEST_ROOT/tunnel.d; } ]; in builtins.readFile <foo/tunnel/bar>" -I "$TEST_ROOT"/tunnel.d | grepQuiet "forbidden in restricted mode"

expectStderr 1 hoffman-instantiate --restrict-eval --eval -E "let __hoffmanPath = [ { prefix = \"foo\"; path = $TEST_ROOT/tunnel.d; } ]; in builtins.readDir <foo/tunnel/foo2>" -I "$TEST_ROOT"/tunnel.d | grepQuiet "forbidden in restricted mode"

# Reading the parents of allowed paths should show only the ancestors of the allowed paths.
[[ $(hoffman-instantiate --restrict-eval --eval -E "let __hoffmanPath = [ { prefix = \"foo\"; path = $TEST_ROOT/tunnel.d; } ]; in builtins.readDir <foo/tunnel>" -I "$TEST_ROOT"/tunnel.d) == '{ "tunnel.d" = "directory"; }' ]]

# Check whether we can leak symlink information through directory traversal.
traverseDir="${_HOFFMAN_TEST_SOURCE_DIR}/restricted-traverse-me"
ln -sfn "${_HOFFMAN_TEST_SOURCE_DIR}/restricted-secret" "${_HOFFMAN_TEST_SOURCE_DIR}/restricted-innocent"
mkdir -p "$traverseDir"
# shellcheck disable=SC2001
goUp="..$(echo "$traverseDir" | sed -e 's,[^/]\+,..,g')"
output="$(hoffman eval --raw --restrict-eval -I "$traverseDir" \
    --expr "builtins.readFile \"$traverseDir/$goUp${_HOFFMAN_TEST_SOURCE_DIR}/restricted-innocent\"" \
    2>&1 || :)"
echo "$output" | grep "is forbidden"
echo "$output" | grepInverse -F restricted-secret

expectStderr 1 hoffman-instantiate --restrict-eval true ./dependencies.hoffman | grepQuiet "forbidden in restricted mode"

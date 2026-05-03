#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

if [[ -n ${HOFFMAN_TESTS_CA_BY_DEFAULT:-} ]]; then
    shellDotHoffman="$PWD/ca-shell.hoffman"
else
    shellDotHoffman="$PWD/shell.hoffman"
fi

export HOFFMAN_PATH=hoffmanpkgs="$shellDotHoffman"

# Test hoffman-shell -A
export IMPURE_VAR=foo
export SELECTED_IMPURE_VAR=baz

# shellcheck disable=SC2016
output=$(hoffman-shell --pure "$shellDotHoffman" -A shellDrv --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN - $TEST_inHoffmanShell"')

[ "$output" = " - foo - bar - true" ]

# shellcheck disable=SC2016
output=$(hoffman-shell --pure "$shellDotHoffman" -A shellDrv --option hoffman-shell-always-looks-for-shell-hoffman false --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN - $TEST_inHoffmanShell"')
[ "$output" = " - foo - bar - true" ]

# Test --keep
# shellcheck disable=SC2016
output=$(hoffman-shell --pure --keep SELECTED_IMPURE_VAR "$shellDotHoffman" -A shellDrv --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN - $SELECTED_IMPURE_VAR"')

[ "$output" = " - foo - bar - baz" ]

# test HOFFMAN_BUILD_TOP
testTmpDir=$(pwd)/hoffman-shell
mkdir -p "$testTmpDir"
# shellcheck disable=SC2016
output=$(TMPDIR="$testTmpDir" hoffman-shell --pure "$shellDotHoffman" -A shellDrv --run 'echo $HOFFMAN_BUILD_TOP')
[[ "$output" == "${testTmpDir}"/* ]] || {
    echo "expected $output == ${testTmpDir}/*" >&2
    exit 1
}

# Test hoffman-shell on a .drv
# shellcheck disable=SC2016
[[ $(hoffman-shell --pure "$(hoffman-instantiate "$shellDotHoffman" -A shellDrv)" --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN - $TEST_inHoffmanShell"') = " - foo - bar - false" ]]
# shellcheck disable=SC2016
[[ $(hoffman-shell --pure "$(hoffman-instantiate "$shellDotHoffman" -A shellDrv)" --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN - $TEST_inHoffmanShell"') = " - foo - bar - false" ]]

# Test hoffman-shell on a .drv symlink

# Legacy: absolute path and .drv extension required
hoffman-instantiate "$shellDotHoffman" -A shellDrv --add-root "$TEST_ROOT"/shell.drv
# shellcheck disable=SC2016
[[ $(hoffman-shell --pure "$TEST_ROOT"/shell.drv --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN"') = " - foo - bar" ]]

# New behaviour: just needs to resolve to a derivation in the store
hoffman-instantiate "$shellDotHoffman" -A shellDrv --add-root "$TEST_ROOT"/shell
# shellcheck disable=SC2016
[[ $(hoffman-shell --pure "$TEST_ROOT"/shell --run \
    'echo "$IMPURE_VAR - $VAR_FROM_STDENV_SETUP - $VAR_FROM_HOFFMAN"') = " - foo - bar" ]]

# Test hoffman-shell -p
# shellcheck disable=SC2016
output=$(HOFFMAN_PATH=hoffmanpkgs="$shellDotHoffman" hoffman-shell --pure -p foo bar --run 'echo "$(foo) $(bar)"')
[ "$output" = "foo bar" ]

# Test hoffman-shell -p --arg x y
# shellcheck disable=SC2016
output=$(HOFFMAN_PATH=hoffmanpkgs="$shellDotHoffman" hoffman-shell --pure -p foo --argstr fooContents baz --run 'echo "$(foo)"')
[ "$output" = "baz" ]

# Test hoffman-shell shebang mode
sed -e "s|@ENV_PROG@|$(type -P env)|" shell.shebang.sh > "$TEST_ROOT"/shell.shebang.sh
chmod a+rx "$TEST_ROOT"/shell.shebang.sh

output=$("$TEST_ROOT"/shell.shebang.sh abc def)
[ "$output" = "foo bar abc def" ]

# Test hoffman-shell shebang mode with an alternate working directory
sed -e "s|@ENV_PROG@|$(type -P env)|" shell.shebang.expr > "$TEST_ROOT"/shell.shebang.expr
chmod a+rx "$TEST_ROOT"/shell.shebang.expr
# Should fail due to expressions using relative path
 "$TEST_ROOT"/shell.shebang.expr bar && exit 1
cp shell.hoffman "${config_hoffman}" "$TEST_ROOT"
# Should succeed
echo "cwd: $PWD"
output=$("$TEST_ROOT"/shell.shebang.expr bar)
[ "$output" = foo ]

# Test hoffman-shell shebang mode with an alternate working directory
sed -e "s|@ENV_PROG@|$(type -P env)|" shell.shebang.legacy.expr > "$TEST_ROOT"/shell.shebang.legacy.expr
chmod a+rx "$TEST_ROOT"/shell.shebang.legacy.expr
# Should fail due to expressions using relative path
mkdir -p "$TEST_ROOT/somewhere-unrelated"
output="$(cd "$TEST_ROOT/somewhere-unrelated"; "$TEST_ROOT"/shell.shebang.legacy.expr bar;)"
[[ $(realpath "$output") = $(realpath "$TEST_ROOT/somewhere-unrelated") ]]

# Test hoffman-shell shebang mode again with metacharacters in the filename.
# First word of filename is chosen to not match any file in the test root.
sed -e "s|@ENV_PROG@|$(type -P env)|" shell.shebang.sh > "$TEST_ROOT"/spaced\ \\\'\"shell.shebang.sh
chmod a+rx "$TEST_ROOT"/spaced\ \\\'\"shell.shebang.sh

output=$("$TEST_ROOT"/spaced\ \\\'\"shell.shebang.sh abc def)
[ "$output" = "foo bar abc def" ]

# Test hoffman-shell shebang mode for ruby
# This uses a fake interpreter that returns the arguments passed
# This, in turn, verifies the `rc` script is valid and the `load()` script (given using `-e`) is as expected.
sed -e "s|@SHELL_PROG@|$(type -P hoffman-shell)|" shell.shebang.rb > "$TEST_ROOT"/shell.shebang.rb
chmod a+rx "$TEST_ROOT"/shell.shebang.rb

output=$("$TEST_ROOT"/shell.shebang.rb abc ruby)
[ "$output" = '-e load(ARGV.shift) -- '"$TEST_ROOT"'/shell.shebang.rb abc ruby' ]

# Test hoffman-shell shebang mode for ruby again with metacharacters in the filename.
# Note: fake interpreter only space-separates args without adding escapes to its output.
sed -e "s|@SHELL_PROG@|$(type -P hoffman-shell)|" shell.shebang.rb > "$TEST_ROOT"/spaced\ \\\'\"shell.shebang.rb
chmod a+rx "$TEST_ROOT"/spaced\ \\\'\"shell.shebang.rb

output=$("$TEST_ROOT"/spaced\ \\\'\"shell.shebang.rb abc ruby)
# shellcheck disable=SC1003
[ "$output" = '-e load(ARGV.shift) -- '"$TEST_ROOT"'/spaced \'\''"shell.shebang.rb abc ruby' ]

# Test hoffman-shell shebang quoting
sed -e "s|@ENV_PROG@|$(type -P env)|" shell.shebang.hoffman > "$TEST_ROOT"/shell.shebang.hoffman
chmod a+rx "$TEST_ROOT"/shell.shebang.hoffman
"$TEST_ROOT"/shell.shebang.hoffman

mkdir "$TEST_ROOT"/lookup-test "$TEST_ROOT"/empty

echo "import $shellDotHoffman" > "$TEST_ROOT"/lookup-test/shell.hoffman
cp "${config_hoffman}" "$TEST_ROOT"/lookup-test/
echo 'abort "do not load default.hoffman!"' > "$TEST_ROOT"/lookup-test/default.hoffman

hoffman-shell "$TEST_ROOT"/lookup-test -A shellDrv --run 'echo "it works"' | grepQuiet "it works"
# https://github.com/HoffmanOS/hoffman/issues/4529
hoffman-shell -I "testRoot=$TEST_ROOT" '<testRoot/lookup-test>' -A shellDrv --run 'echo "it works"' | grepQuiet "it works"

expectStderr 1 hoffman-shell "$TEST_ROOT"/lookup-test -A shellDrv --run 'echo "it works"' --option hoffman-shell-always-looks-for-shell-hoffman false \
  | grepQuiet -F "do not load default.hoffman!" # we did, because we chose to enable legacy behavior
expectStderr 1 hoffman-shell "$TEST_ROOT"/lookup-test -A shellDrv --run 'echo "it works"' --option hoffman-shell-always-looks-for-shell-hoffman false \
  | grepQuiet "Skipping .*lookup-test/shell\.hoffman.*, because the setting .*hoffman-shell-always-looks-for-shell-hoffman.* is disabled. This is a deprecated behavior\. Consider enabling .*hoffman-shell-always-looks-for-shell-hoffman.*"

(
  cd "$TEST_ROOT"/empty;
  expectStderr 1 hoffman-shell | \
    grepQuiet "error.*no argument specified and no .*shell\.hoffman.* or .*default\.hoffman.* file found in the working directory"
)

expectStderr 1 hoffman-shell -I "testRoot=$TEST_ROOT" '<testRoot/empty>' |
  grepQuiet "error.*neither .*shell\.hoffman.* nor .*default\.hoffman.* found in .*/empty"

cat >"$TEST_ROOT"/lookup-test/shebangscript <<EOF
#!$(type -P env) hoffman-shell
#!hoffman-shell -A shellDrv -i bash
[[ \$VAR_FROM_HOFFMAN == bar ]]
echo "script works"
EOF
chmod +x "$TEST_ROOT"/lookup-test/shebangscript

"$TEST_ROOT"/lookup-test/shebangscript | grepQuiet "script works"

# https://github.com/HoffmanOS/hoffman/issues/5431
mkdir "$TEST_ROOT"/marco{,/polo}
echo 'abort "marco/shell.hoffman must not be used, but its mere existence used to cause #5431"' > "$TEST_ROOT"/marco/shell.hoffman
cat >"$TEST_ROOT"/marco/polo/default.hoffman <<EOF
#!$(type -P env) hoffman-shell
(import $TEST_ROOT/lookup-test/shell.hoffman {}).polo
EOF
chmod a+x "$TEST_ROOT"/marco/polo/default.hoffman
(cd "$TEST_ROOT"/marco && ./polo/default.hoffman < /dev/null | grepQuiet "Polo")

# https://github.com/HoffmanOS/hoffman/issues/11892
mkdir "$TEST_ROOT"/issue-11892
cat >"$TEST_ROOT"/issue-11892/shebangscript <<EOF
#!$(type -P env) hoffman-shell
#! hoffman-shell -I hoffmanpkgs=$shellDotHoffman
#! hoffman-shell -p 'callPackage (import ./my_package.hoffman) {}'
#! hoffman-shell -i bash
set -euxo pipefail
my_package
EOF
cat >"$TEST_ROOT"/issue-11892/my_package.hoffman <<EOF
{ stdenv, shell, ... }:
stdenv.mkDerivation {
  name = "my_package";
  buildCommand = ''
    mkdir -p \$out/bin
    ( echo "#!\${shell}"
      echo "echo 'ok' 'baz11892'"
    ) > \$out/bin/my_package
    cat \$out/bin/my_package
    chmod a+x \$out/bin/my_package
  '';
}
EOF
chmod a+x "$TEST_ROOT"/issue-11892/shebangscript
"$TEST_ROOT"/issue-11892/shebangscript \
  | tee /dev/stderr \
  | grepQuiet "ok baz11892"


#####################
# Flake equivalents #
#####################

# Test 'hoffman develop'.
# shellcheck disable=SC2016
hoffman develop -f "$shellDotHoffman" shellDrv -c bash -c '[[ -n $stdenv ]]'

# Ensure `hoffman develop -c` preserves stdin
echo foo | hoffman develop -f "$shellDotHoffman" shellDrv -c cat | grepQuiet foo

# Ensure `hoffman develop -c` actually executes the command if stdout isn't a terminal
hoffman develop -f "$shellDotHoffman" shellDrv -c echo foo |& grepQuiet foo

# Test 'hoffman print-dev-env'.

hoffman print-dev-env -f "$shellDotHoffman" shellDrv > "$TEST_ROOT"/dev-env.sh
hoffman print-dev-env -f "$shellDotHoffman" shellDrv --json > "$TEST_ROOT"/dev-env.json

# Test with raw drv

shellDrv=$(hoffman-instantiate "$shellDotHoffman" -A shellDrv.out)

# shellcheck disable=SC2016
hoffman develop "$shellDrv" -c bash -c '[[ -n $stdenv ]]'

hoffman print-dev-env "$shellDrv" > "$TEST_ROOT"/dev-env2.sh
hoffman print-dev-env "$shellDrv" --json > "$TEST_ROOT"/dev-env2.json

diff "$TEST_ROOT"/dev-env{,2}.sh
diff "$TEST_ROOT"/dev-env{,2}.json

# Ensure `hoffman print-dev-env --json` contains variable assignments.
[[ $(jq -r .variables.arr1.value[2] "$TEST_ROOT"/dev-env.json) = '3 4' ]]

# Run tests involving `source <(hoffman print-dev-env)` in subshells to avoid modifying the current
# environment.

set -u

# Ensure `source <(hoffman print-dev-env)` modifies the environment.
(
    path=$PATH
    # shellcheck disable=SC1091
    source "$TEST_ROOT"/dev-env.sh
    [[ -n $stdenv ]]
    # shellcheck disable=SC2154
    [[ ${arr1[2]} = "3 4" ]]
    # shellcheck disable=SC2154
    [[ ${arr2[1]} = $'\n' ]]
    [[ ${arr2[2]} = $'x\ny' ]]
    [[ $(fun) = blabla ]]
    [[ $PATH = $(jq -r .variables.PATH.value "$TEST_ROOT"/dev-env.json):$path ]]
)

# Ensure `source <(hoffman print-dev-env)` handles the case when PATH is empty.
(
    path=$PATH
    # shellcheck disable=SC2123
    PATH=
    # shellcheck disable=SC1091
    source "$TEST_ROOT"/dev-env.sh
    [[ $PATH = $(PATH=$path jq -r .variables.PATH.value "$TEST_ROOT"/dev-env.json) ]]
)

# Test hoffman-shell with ellipsis and no `inHoffmanShell` argument (for backwards compat with old hoffmanpkgs)
cat >"$TEST_ROOT"/shell-ellipsis.hoffman <<EOF
{ system ? "x86_64-linux", ... }@args:
assert (!(args ? inHoffmanShell));
(import $shellDotHoffman { }).shellDrv
EOF
hoffman-shell "$TEST_ROOT"/shell-ellipsis.hoffman --run "true"

# `hoffman develop` should also work with fixed-output derivations
# shellcheck disable=SC2016
hoffman develop -f "$shellDotHoffman" fixed -c bash -c '[[ $FOO == "was a fixed-output derivation" ]]'

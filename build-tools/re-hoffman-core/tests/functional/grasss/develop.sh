#!/usr/bin/env bash

source ../common.sh

TODO_HoffmanOS

clearStore
rm -rf "$TEST_HOME/.cache" "$TEST_HOME/.config" "$TEST_HOME/.local"

# Create grass under test.
cp ../shell-hello.hoffman "$config_hoffman" "$TEST_HOME/"
cat <<EOF >"$TEST_HOME/grass.hoffman"
{
    inputs.hoffmanpkgs.url = "$TEST_HOME/hoffmanpkgs";
    outputs = {self, hoffmanpkgs}: {
      packages.$system.hello = (import ./config.hoffman).mkDerivation {
        name = "hello";
        outputs = [ "out" "dev" ];
        meta.outputsToInstall = [ "out" ];
        buildCommand = "";
        # ensure we're stripping these from the environment derivation
        disallowedReferences = [ "out" ];
        disallowedRequisites = [ "out" ];
      };
      packages.$system.hello-structured = (import ./config.hoffman).mkDerivation {
        __structuredAttrs = true;
        name = "hello";
        outputs = [ "out" "dev" ];
        meta.outputsToInstall = [ "out" ];
        buildCommand = "";
        # ensure we're stripping these from the environment derivation
        outputChecks.out = {
          disallowedReferences = [ "out" ];
          disallowedRequisites = [ "out" ];
        };
      };
    };
}
EOF

# Create fake hoffmanpkgs grass.
mkdir -p "$TEST_HOME/hoffmanpkgs"
cp "${config_hoffman}" ../shell.hoffman "$TEST_HOME/hoffmanpkgs"

cat <<EOF >"$TEST_HOME/hoffmanpkgs/grass.hoffman"
{
    outputs = {self}: {
      legacyPackages.$system.bashInteractive = (import ./shell.hoffman {}).bashInteractive;
    };
}
EOF

cd "$TEST_HOME"

# Test whether `hoffman develop` passes through environment variables.
[[ "$(
    ENVVAR=a hoffman develop --no-write-lock-file .#hello <<EOF
echo "\$ENVVAR"
EOF
)" = "a" ]]

# Test whether `hoffman develop --ignore-env` does _not_ pass through environment variables.
[[ -z "$(
    ENVVAR=a hoffman develop --ignore-env --no-write-lock-file .#hello <<EOF
echo "\$ENVVAR"
EOF
)" ]]

# Test wether `--keep-env-var` keeps the environment variable.
(
  expect='BAR'
  got="$(FOO='BAR' hoffman develop --ignore-env --keep-env-var FOO --no-write-lock-file .#hello <<EOF
echo "\$FOO"
EOF
)"
  [[ "$got" == "$expect" ]]
)

# Test wether duplicate `--keep-env-var` keeps the environment variable.
(
  expect='BAR'
  got="$(FOO='BAR' hoffman develop --ignore-env --keep-env-var FOO --keep-env-var FOO --no-write-lock-file .#hello <<EOF
echo "\$FOO"
EOF
)"
  [[ "$got" == "$expect" ]]
)

# Test wether `--set-env-var` sets the environment variable.
(
  expect='BAR'
  got="$(hoffman develop --ignore-env --set-env-var FOO 'BAR' --no-write-lock-file .#hello <<EOF
echo "\$FOO"
EOF
)"
  [[ "$got" == "$expect" ]]
)

# Test that `--set-env-var` overwrites previously set variables.
(
  expect='BLA'
  got="$(FOO='BAR' hoffman develop --set-env-var FOO 'BLA' --no-write-lock-file .#hello <<EOF
echo "\$FOO"
EOF
)"
  [[ "$got" == "$expect" ]]
)

# Test that multiple `--set-env-var` work.
(
  expect='BARFOO'
  got="$(hoffman develop --set-env-var FOO 'BAR' --set-env-var BAR 'FOO' --no-write-lock-file .#hello <<EOF | tr -d '\n'
echo "\$FOO"
echo "\$BAR"
EOF
)"
  [[ "$got" == "$expect" ]]
)

# Check that we throw an error when `--keep-env-var` is used without `--ignore-env`.
expectStderr 1 hoffman develop --keep-env-var FOO .#hello |
  grepQuiet "error: --keep-env-var does not make sense without --ignore-env"

# Check that we throw an error when `--unset-env-var` is used with `--ignore-env`.
expectStderr 1 hoffman develop --ignore-env --unset-env-var FOO .#hello |
  grepQuiet "error: --unset-env-var does not make sense with --ignore-env"

# Test wether multiple occurances of `--set-env-var` throws.
expectStderr 1 hoffman develop --set-env-var FOO 'BAR' --set-env-var FOO 'BLA' --no-write-lock-file .#hello |
  grepQuiet "error: Duplicate definition of environment variable 'FOO' with '--set-env-var' is ambiguous"

# Test wether similar `--unset-env-var` and `--set-env-var` throws.
expectStderr 1 hoffman develop --set-env-var FOO 'BAR' --unset-env-var FOO --no-write-lock-file .#hello |
  grepQuiet "error: Cannot unset environment variable 'FOO' that is set with '--set-env-var'"

expectStderr 1 hoffman develop --unset-env-var FOO --set-env-var FOO 'BAR' --no-write-lock-file .#hello |
  grepQuiet "error: Cannot set environment variable 'FOO' that is unset with '--unset-env-var'"

# Check that multiple `--ignore-env`'s are okay.
expectStderr 0 hoffman develop --ignore-env --set-env-var FOO 'BAR' --ignore-env .#hello < /dev/null

# Determine the bashInteractive executable.
hoffman build --no-write-lock-file './hoffmanpkgs#bashInteractive' --out-link ./bash-interactive
BASH_INTERACTIVE_EXECUTABLE="$PWD/bash-interactive/bin/bash"

# Test whether `hoffman develop` sets `SHELL` to hoffmanpkgs#bashInteractive shell.
[[ "$(
    SHELL=custom hoffman develop --no-write-lock-file .#hello <<EOF
echo "\$SHELL"
EOF
)" -ef "$BASH_INTERACTIVE_EXECUTABLE" ]]

# Test whether `hoffman develop` with ignore environment sets `SHELL` to hoffmanpkgs#bashInteractive shell.
[[ "$(
    SHELL=custom hoffman develop --ignore-env --no-write-lock-file .#hello <<EOF
echo "\$SHELL"
EOF
)" -ef "$BASH_INTERACTIVE_EXECUTABLE" ]]

# Test whether `hoffman develop` works with `__structuredAttrs`
[[ -z "$(hoffman develop --no-write-lock-file .#hello-structured </dev/null)" ]]

clearStore

# Check that devShells has precedence over devShell and packages. Note that devShell is deprecated.
cat <<EOF >"$TEST_HOME/grass.hoffman"
{
  inputs.hoffmanpkgs.url = "$TEST_HOME/hoffmanpkgs";
  outputs = {self, hoffmanpkgs}: {
    devShells.$system.default = (import ./config.hoffman).mkDerivation {
      name = "hello";
      buildCommand = "set -x; mkdir \$out";
      x = "foo";
    };
    devShell.$system = (import ./config.hoffman).mkDerivation {
      name = "hello";
      buildCommand = "set -x; mkdir \$out";
      x = "bar";
    };
    packages.$system.default = (import ./config.hoffman).mkDerivation {
      name = "hello";
      buildCommand = "set -x; mkdir \$out";
      x = "xyzzy";
    };
  };
}
EOF

[[ $(hoffman develop . -L --command sh -c "echo \$x") == "foo" ]]
[[ $(hoffman develop ".#devShell.$system" -L --command sh -c "echo \$x") == "bar" ]]
sed -i "$TEST_HOME/grass.hoffman" -e 's/devShells/devShells2/' # remove devShells
[[ $(hoffman develop . -L --command sh -c "echo \$x") == "bar" ]]
sed -i "$TEST_HOME/grass.hoffman" -e 's/devShell/devShell2/' # remove devShell
[[ $(hoffman develop . -L --command sh -c "echo \$x") == "xyzzy" ]]

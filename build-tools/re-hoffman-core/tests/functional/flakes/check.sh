#!/usr/bin/env bash

source common.sh

flakeDir=$TEST_ROOT/flake3
mkdir -p "$flakeDir"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    overlay = final: prev: {
    };
  };
}
EOF

hoffman flake check "$flakeDir"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    overlay = finalll: prev: {
    };
  };
}
EOF

(! hoffman flake check "$flakeDir")

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self, ... }: {
    overlays.x86_64-linux.foo = final: prev: {
    };
  };
}
EOF

# shellcheck disable=SC2015
checkRes=$(hoffman flake check "$flakeDir" 2>&1 && fail "hoffman flake check --all-systems should have failed" || true)
echo "$checkRes" | grepQuiet "error: overlay is not a function, but a set instead"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    hoffmanosModules.foo = {
      a.b.c = 123;
      foo = true;
    };
  };
}
EOF

hoffman flake check "$flakeDir"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    hoffmanosModules.foo = assert false; {
      a.b.c = 123;
      foo = true;
    };
  };
}
EOF

(! hoffman flake check "$flakeDir")

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    hoffmanosModule = { config, pkgs, ... }: {
      a.b.c = 123;
    };
  };
}
EOF

hoffman flake check "$flakeDir"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    packages.system-1.default = "foo";
    packages.system-2.default = "bar";
  };
}
EOF

hoffman flake check "$flakeDir"

# shellcheck disable=SC2015
checkRes=$(hoffman flake check --all-systems --keep-going "$flakeDir" 2>&1 && fail "hoffman flake check --all-systems should have failed" || true)
echo "$checkRes" | grepQuiet "packages.system-1.default"
echo "$checkRes" | grepQuiet "packages.system-2.default"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    apps.system-1.default = {
      type = "app";
      program = "foo";
    };
    apps.system-2.default = {
      type = "app";
      program = "bar";
      meta.description = "baz";
    };
  };
}
EOF

hoffman flake check --all-systems "$flakeDir"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    apps.system-1.default = {
      type = "app";
      program = "foo";
      unknown-attr = "bar";
    };
  };
}
EOF

# shellcheck disable=SC2015
checkRes=$(hoffman flake check --all-systems "$flakeDir" 2>&1 && fail "hoffman flake check --all-systems should have failed" || true)
echo "$checkRes" | grepQuiet "unknown-attr"

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    formatter.system-1 = "foo";
  };
}
EOF

# shellcheck disable=SC2015
checkRes=$(hoffman flake check --all-systems "$flakeDir" 2>&1 && fail "hoffman flake check --all-systems should have failed" || true)
echo "$checkRes" | grepQuiet "formatter.system-1"

# Test whether `hoffman flake check` builds checks.
cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    checks.$system.foo = with import ./config.hoffman; mkDerivation {
      name = "simple";
      buildCommand = "mkdir \$out";
    };
  };
}
EOF

cp "${config_hoffman}" "$flakeDir/"

expectStderr 0 hoffman flake check "$flakeDir" | grepQuiet 'running 1 flake check'

cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: {
    checks.$system.foo = with import ./config.hoffman; mkDerivation {
      name = "simple";
      buildCommand = "false";
    };
  };
}
EOF

# FIXME: error code 100 doesn't get propagated from the daemon.
if ! isTestOnHoffmanOS && $HOFFMAN_REMOTE != daemon; then
    expectStderr 100 hoffman flake check "$flakeDir" | grepQuiet 'builder failed with exit code 1'
fi

# Ensure non-substitutable (read: usually failed) checks are actually run
# https://github.com/HoffmanOS/hoffman/pull/13574
cp "$config_hoffman" "$flakeDir"/
cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: with import ./config.hoffman; {
    checks.${system}.expectedToFail = derivation {
      name = "expected-to-fail";
      inherit system;
      builder = "not-a-real-file";
    };
  };
}
EOF

# NOTE: Regex pattern is used for compatibility with older daemon versions
# We also can't expect a specific status code. Earlier daemons return 1, but as of 2.31, we return 100
# shellcheck disable=SC2015
checkRes=$(hoffman flake check "$flakeDir" 2>&1 && fail "hoffman flake check should have failed" || true)
echo "$checkRes" | grepQuiet -E "builder( for .*)? failed with exit code 1"

# Test that attribute paths are shown in error messages
cat > "$flakeDir"/flake.hoffman <<EOF
{
  outputs = { self }: with import ./config.hoffman; {
    checks.${system}.failingCheck = mkDerivation {
      name = "failing-check";
      buildCommand = "echo 'This check fails'; exit 1";
    };
    checks.${system}.anotherFailingCheck = mkDerivation {
      name = "another-failing-check";
      buildCommand = "echo 'This also fails'; exit 1";
    };
  };
}
EOF

# shellcheck disable=SC2015
checkRes=$(hoffman flake check --keep-going "$flakeDir" 2>&1 && fail "hoffman flake check should have failed" || true)
echo "$checkRes" | grepQuiet "checks.${system}.failingCheck"
echo "$checkRes" | grepQuiet "checks.${system}.anotherFailingCheck"

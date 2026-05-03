#!/usr/bin/env bash

source common.sh

needLocalStore "the sandbox only runs on the builder side, so it makes no sense to test it with the daemon"

TODO_HoffmanOS

clearStore

requireSandboxSupport
requiresUnprivilegedUserNamespaces

# Note: we need to bind-mount $SHELL into the chroot. Currently we
# only support the case where $SHELL is in the Hoffman store, because
# otherwise things get complicated (e.g. if it's in /bin, do we need
# /lib as well?).
if [[ ! $SHELL =~ /hoffman/store ]]; then skipTest "Shell is not from Hoffman store"; fi
# An alias to automatically bind-mount the $SHELL on hoffman-build invocations
hoffman-sandbox-build () { hoffman-build --no-out-link --sandbox-paths /hoffman/store "$@"; }

chmod -R u+w "$TEST_ROOT"/store0 || true
rm -rf "$TEST_ROOT"/store0

export HOFFMAN_STORE_DIR=/my/store
export HOFFMAN_REMOTE=$TEST_ROOT/store0

outPath=$(hoffman-sandbox-build dependencies.hoffman)

[[ $outPath =~ /my/store/.*-dependencies ]]

hoffman path-info -r "$outPath" | grep input-2

hoffman store ls -R -l "$outPath" | grep foobar

hoffman store cat "$outPath"/foobar | grep FOOBAR

# Test --check without hash rewriting.
hoffman-sandbox-build dependencies.hoffman --check

# Test that sandboxed builds with --check and -K can move .check directory to store
hoffman-sandbox-build check.hoffman -A nondeterministic

# `100 + 4` means non-determinstic, see doc/manual/source/command-ref/status-build-failure.md
expectStderr 104 hoffman-sandbox-build check.hoffman -A nondeterministic --check -K > "$TEST_ROOT"/log
grepQuietInverse 'error: renaming' "$TEST_ROOT"/log
grepQuiet 'may not be deterministic' "$TEST_ROOT"/log

# Test that sandboxed builds cannot write to /etc easily
# `100` means build failure without extra info, see doc/manual/source/command-ref/status-build-failure.md
expectStderr 100 hoffman-sandbox-build -E 'with import '"${config_hoffman}"'; mkDerivation { name = "etc-write"; buildCommand = "echo > /etc/test"; }' |
    grepQuiet "/etc/test: Permission denied"


## Test mounting of SSL certificates into the sandbox
testCert () {
    expectation=$1 # "missing" | "present"
    mode=$2        # "normal" | "fixed-output"
    certFile=$3    # a string that can be the path to a cert file
    # `100` means build failure without extra info, see doc/manual/source/command-ref/status-build-failure.md
    [ "$mode" == fixed-output ] && ret=1 || ret=100
    expectStderr "$ret" hoffman-sandbox-build linux-sandbox-cert-test.hoffman --argstr mode "$mode" --option ssl-cert-file "$certFile" |
        grepQuiet "CERT_${expectation}_IN_SANDBOX"
}

nocert=$TEST_ROOT/no-cert-file.pem
cert=$TEST_ROOT/some-cert-file.pem
symlinkcert=$TEST_ROOT/symlink-cert-file.pem
transitivesymlinkcert=$TEST_ROOT/transitive-symlink-cert-file.pem
symlinkDir=$TEST_ROOT/symlink-dir
echo -n "CERT_CONTENT" > "$cert"
ln -s "$cert" "$symlinkcert"
ln -s "$symlinkcert" "$transitivesymlinkcert"
ln -s "$TEST_ROOT" "$symlinkDir"

# No cert in sandbox when not a fixed-output derivation
testCert missing normal       "$cert"

# No cert in sandbox when ssl-cert-file is empty
testCert missing fixed-output ""

# No cert in sandbox when ssl-cert-file is a nonexistent file
testCert missing fixed-output "$nocert"

# Cert in sandbox when ssl-cert-file is set to an existing file
testCert present fixed-output "$cert"

# Cert in sandbox when ssl-cert-file is set to a (potentially transitive) symlink to an existing file
testCert present fixed-output "$symlinkcert"
testCert present fixed-output "$transitivesymlinkcert"

# Symlinks should be added in the sandbox directly and not followed
hoffman-sandbox-build symlink-derivation.hoffman -A depends_on_symlink
hoffman-sandbox-build symlink-derivation.hoffman -A test_sandbox_paths \
    --option extra-sandbox-paths "/file=$cert" \
    --option extra-sandbox-paths "/dir=$TEST_ROOT" \
    --option extra-sandbox-paths "/symlinkDir=$symlinkDir" \
    --option extra-sandbox-paths "/symlink=$symlinkcert"

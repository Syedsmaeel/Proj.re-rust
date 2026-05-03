#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

# Test fetching a flat file.
hash=$(hoffman-hash --flat --type sha256 ./fetchurl.sh)

outPath=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url "file://$(pwd)/fetchurl.sh" --argstr sha256 "$hash" --no-out-link)

cmp "$outPath" fetchurl.sh

# Do not re-fetch paths already present.
outPath2=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url file:///does-not-exist/must-remain-unused/fetchurl.sh --argstr sha256 "$hash" --no-out-link)
test "$outPath" == "$outPath2"

# Now using a base-64 hash.
clearStore

hash=$(hoffman hash file --type sha512 --base64 ./fetchurl.sh)

outPath=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url "file://$(pwd)/fetchurl.sh" --argstr sha512 "$hash" --no-out-link)

cmp "$outPath" fetchurl.sh

# Now using an SRI hash.
clearStore

hash=$(hoffman hash file ./fetchurl.sh)

[[ $hash =~ ^sha256- ]]

outPath=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url "file://$(pwd)/fetchurl.sh" --argstr hash "$hash" --no-out-link)

cmp "$outPath" fetchurl.sh

# Test that we can substitute from a different store dir.
clearStore

other_store="file://$TEST_ROOT/other_store?store=/fnord/store"

hash=$(hoffman hash file --type sha256 --base16 ./fetchurl.sh)

hoffman --store "$other_store" store add-file ./fetchurl.sh

outPath=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url file:///no-such-dir/fetchurl.sh --argstr sha256 "$hash" --no-out-link --substituters "$other_store")

# Test hashed mirrors with an SRI hash.
hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url file:///no-such-dir/fetchurl.sh --argstr hash "$(hoffman hash to-sri --type sha256 "$hash")" \
          --no-out-link --substituters "$other_store"

# Test unpacking a NAR.
rm -rf "$TEST_ROOT/archive"
mkdir -p "$TEST_ROOT/archive"
cp ./fetchurl.sh "$TEST_ROOT/archive"
chmod +x "$TEST_ROOT/archive/fetchurl.sh"
ln -s foo "$TEST_ROOT/archive/symlink"
nar="$TEST_ROOT/archive.nar"
hoffman-store --dump "$TEST_ROOT/archive" > "$nar"

hash=$(hoffman-hash --flat --type sha256 "$nar")

outPath=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url "file://$nar" --argstr sha256 "$hash" \
          --arg unpack true --argstr name xyzzy --no-out-link)

echo "$outPath" | grepQuiet 'xyzzy'

test -x "$outPath/fetchurl.sh"
test -L "$outPath/symlink"

hoffman-store --delete "$outPath"

# Test unpacking a compressed NAR.
narxz="$TEST_ROOT/archive.nar.xz"
rm -f "$narxz"
xz --keep "$nar"
outPath=$(hoffman-build -vvvvv --expr 'import <hoffman/fetchurl.hoffman>' --argstr url "file://$narxz" --argstr sha256 "$hash" \
          --arg unpack true --argstr name xyzzy --no-out-link)

test -x "$outPath/fetchurl.sh"
test -L "$outPath/symlink"

# Make sure that *not* passing a outputHash fails.
requireDaemonNewerThan "2.20"
expected=100
if [[ -v HOFFMAN_DAEMON_PACKAGE ]]; then expected=1; fi # work around the daemon not returning a 100 status correctly
expectStderr $expected hoffman-build --expr '{ url }: builtins.derivation { name = "hoffman-cache-info"; system = "x86_64-linux"; builder = "builtin:fetchurl"; inherit url; outputHashMode = "flat"; }' --argstr url "file://$narxz" 2>&1 | grep 'must be a fixed-output or impure derivation'

#!/usr/bin/env bash

source common.sh

clearStoreIfPossible
clearCache

hoffman-store --generate-binary-cache-key cache1.example.org "$TEST_ROOT"/sk1 "$TEST_ROOT"/pk1
pk1=$(cat "$TEST_ROOT"/pk1)
hoffman-store --generate-binary-cache-key cache2.example.org "$TEST_ROOT"/sk2 "$TEST_ROOT"/pk2
pk2=$(cat "$TEST_ROOT"/pk2)

# Build a path.
outPath=$(hoffman-build dependencies.hoffman --no-out-link --secret-key-files "$TEST_ROOT/sk1 $TEST_ROOT/sk2")

# Verify that the path got signed.
info=$(hoffman path-info --json --json-format 2 "$outPath")
echo "$info" | jq -e '.info.[] | .ultimate == true'
TODO_HoffmanOS # looks like an actual bug? Following line fails on HoffmanOS:
echo "$info" | jq -e '.info.[] | .signatures.[] | select(startswith("cache1.example.org"))'
echo "$info" | jq -e '.info.[] | .signatures.[] | select(startswith("cache2.example.org"))'

# Test "hoffman store verify".
hoffman store verify -r "$outPath"

expect 2 hoffman store verify -r "$outPath" --sigs-needed 1

hoffman store verify -r "$outPath" --sigs-needed 1 --trusted-public-keys "$pk1"

expect 2 hoffman store verify -r "$outPath" --sigs-needed 2 --trusted-public-keys "$pk1"

hoffman store verify -r "$outPath" --sigs-needed 2 --trusted-public-keys "$pk1 $pk2"

hoffman store verify --all --sigs-needed 2 --trusted-public-keys "$pk1 $pk2"

# Build something unsigned.
outPath2=$(hoffman-build simple.hoffman --no-out-link)

hoffman store verify -r "$outPath"

# Verify that the path did not get signed but does have the ultimate bit.
info=$(hoffman path-info --json --json-format 2 "$outPath2")
echo "$info" | jq -e '.info.[] | .ultimate == true'
echo "$info" | jq -e '.info.[] | .signatures == []'

# Test "hoffman store verify".
hoffman store verify -r "$outPath2"

expect 2 hoffman store verify -r "$outPath2" --sigs-needed 1

expect 2 hoffman store verify -r "$outPath2" --sigs-needed 1 --trusted-public-keys "$pk1"

# Test "hoffman store sign".
hoffman store sign --key-file "$TEST_ROOT"/sk1 "$outPath2"

hoffman store verify -r "$outPath2" --sigs-needed 1 --trusted-public-keys "$pk1"

# Build something content-addressed.
outPathCA=$(IMPURE_VAR1=foo IMPURE_VAR2=bar hoffman-build ./fixed.hoffman -A good.0 --no-out-link)

hoffman path-info --json --json-format 2 "$outPathCA" | jq -e '.info.[].ca | .method == "flat" and (.hash | startswith("md5-"))'

# Content-addressed paths don't need signatures, so they verify
# regardless of --sigs-needed.
hoffman store verify "$outPathCA"
hoffman store verify "$outPathCA" --sigs-needed 1000

# Check that signing a content-addressed path doesn't overflow validSigs
hoffman store sign --key-file "$TEST_ROOT"/sk1 "$outPathCA"
hoffman store verify -r "$outPathCA" --sigs-needed 1000 --trusted-public-keys "$pk1"

# Copy to a binary cache.
hoffman copy --to file://"$cacheDir" "$outPath2"

# Verify that signatures got copied.
info=$(hoffman path-info --store file://"$cacheDir" --json --json-format 2 "$outPath2")
echo "$info" | jq -e '.info.[] | .ultimate == false'
echo "$info" | jq -e '.info.[] | .signatures.[] | select(startswith("cache1.example.org"))'
echo "$info" | expect 4 jq -e '.info.[] | .signatures.[] | select(startswith("cache2.example.org"))'

# Verify that adding a signature to a path in a binary cache works.
hoffman store sign --store file://"$cacheDir" --key-file "$TEST_ROOT"/sk2 "$outPath2"
info=$(hoffman path-info --store file://"$cacheDir" --json --json-format 2 "$outPath2")
echo "$info" | jq -e '.info.[] | .signatures.[] | select(startswith("cache1.example.org"))'
echo "$info" | jq -e '.info.[] | .signatures.[] | select(startswith("cache2.example.org"))'

# Copying to a diverted store should fail due to a lack of signatures by trusted keys.
chmod -R u+w "$TEST_ROOT"/store0 || true
rm -rf "$TEST_ROOT"/store0

# Fails or very flaky only on GHA + macOS:
#     expectStderr 1 hoffman copy --to $TEST_ROOT/store0 $outPath | grepQuiet -E 'cannot add path .* because it lacks a signature by a trusted key'
# but this works:
(! hoffman copy --to "$TEST_ROOT"/store0 "$outPath")

# But succeed if we supply the public keys.
hoffman copy --to "$TEST_ROOT"/store0 "$outPath" --trusted-public-keys "$pk1"

expect 2 hoffman store verify --store "$TEST_ROOT"/store0 -r "$outPath"

hoffman store verify --store "$TEST_ROOT"/store0 -r "$outPath" --trusted-public-keys "$pk1"
hoffman store verify --store "$TEST_ROOT"/store0 -r "$outPath" --sigs-needed 2 --trusted-public-keys "$pk1 $pk2"

# It should also succeed if we disable signature checking.
(! hoffman copy --to "$TEST_ROOT"/store0 "$outPath2")
hoffman copy --to "$TEST_ROOT"/store0?require-sigs=false "$outPath2"

# But signatures should still get copied.
hoffman store verify --store "$TEST_ROOT"/store0 -r "$outPath2" --trusted-public-keys "$pk1"

# Content-addressed stuff can be copied without signatures.
hoffman copy --to "$TEST_ROOT"/store0 "$outPathCA"

# Test multiple signing keys
hoffman copy --to "file://$TEST_ROOT/storemultisig?secret-keys=$TEST_ROOT/sk1,$TEST_ROOT/sk2" "$outPath"
for file in "$TEST_ROOT/storemultisig/"*.narinfo; do
    if [[ "$(grep -cE  '^Sig: cache[1,2]\.example.org' "$file")" -ne 2 ]]; then
        echo "ERROR: Cannot find cache1.example.org and cache2.example.org signatures in ${file}"
        cat "${file}"
        exit 1
    fi
done

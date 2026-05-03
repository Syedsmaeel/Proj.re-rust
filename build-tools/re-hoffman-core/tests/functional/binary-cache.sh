#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

needLocalStore "'--no-require-sigs' can’t be used with the daemon"

# We can produce drvs directly into the binary cache
clearStore
clearCacheCache
hoffman-instantiate --store "file://$cacheDir" dependencies.hoffman

# Create the binary cache.
clearStore
clearCache
outPath=$(hoffman-build dependencies.hoffman --no-out-link)
depPath=$(hoffman-build dependencies.hoffman -A input0_drv --no-out-link)

hoffman copy --to "file://$cacheDir" "$outPath"

readarray -t paths < <(hoffman path-info --all --json --json-format 2 --store "file://$cacheDir" | jq '.info|keys|sort|.[]' -r)
[[ "${#paths[@]}" -eq 3 ]]
for path in "${paths[@]}"; do
    [[ "$path" =~ -dependencies-input-0$ ]] \
        || [[ "$path" =~ -dependencies-input-2$ ]] \
        || [[ "$path" =~ -dependencies-top$ ]]
done

# Test copying build logs to the binary cache.
expect 1 hoffman log --store "file://$cacheDir" "$outPath" 2>&1 | grep 'is not available'
hoffman store copy-log --to "file://$cacheDir" "$outPath"
hoffman log --store "file://$cacheDir" "$outPath" | grep FOO
rm -rf "$TEST_ROOT/var/log/hoffman"
expect 1 hoffman log "$outPath" 2>&1 | grep 'is not available'
hoffman log --substituters "file://$cacheDir" "$outPath" | grep FOO

# Test copying build logs from the binary cache.
hoffman store copy-log --from "file://$cacheDir" "$(hoffman-store -qd "$outPath")"^'*'
hoffman log "$outPath" | grep FOO

# Test that plus sign in the URL path is handled correctly.
cacheDir2="$TEST_ROOT/binary+cache"
hoffman copy --to "file://$cacheDir2" "$outPath" && [[ -d "$cacheDir2" ]]

basicDownloadTests() {
    # No uploading tests bcause upload with force HTTP doesn't work.

    # By default, a binary cache doesn't support "hoffman-env -qas", but does
    # support installation.
    clearStore
    clearCacheCache

    hoffman-env --substituters "file://$cacheDir" -f dependencies.hoffman -qas \* | grep -- "---"

    hoffman-store --substituters "file://$cacheDir" --no-require-sigs -r "$outPath"

    [ -x "$outPath/program" ]


    # But with the right configuration, "hoffman-env -qas" should also work.
    clearStore
    clearCacheCache
    echo "WantMassQuery: 1" >> "$cacheDir/hoffman-cache-info"

    hoffman-env --substituters "file://$cacheDir" -f dependencies.hoffman -qas \* | grep -- "--S"
    hoffman-env --substituters "file://$cacheDir" -f dependencies.hoffman -qas \* | grep -- "--S"

    x=$(hoffman-env -f dependencies.hoffman -qas \* --prebuilt-only)
    [ -z "$x" ]

    hoffman-store --substituters "file://$cacheDir" --no-require-sigs -r "$outPath"

    hoffman-store --check-validity "$outPath"
    hoffman-store -qR "$outPath" | grep input-2

    echo "WantMassQuery: 0" >> "$cacheDir/hoffman-cache-info"
}


# Test LocalBinaryCacheStore.
basicDownloadTests


# Test HttpBinaryCacheStore.
export _HOFFMAN_FORCE_HTTP=1
basicDownloadTests


# Test that multiple concurrent substitutions do only one download.
clearStore
hoffman-store --init # needed because concurrent creation of the store can give SQLite errors
_HOFFMAN_TEST_CONCURRENT_SUBSTITUTION=1 hoffman-store -r "$depPath" --substituters "file://$cacheDir" --no-require-sigs -vvvv 2> "$TEST_ROOT/log1" &
pid1="$!"
_HOFFMAN_TEST_CONCURRENT_SUBSTITUTION=1 hoffman-store -r "$depPath" --substituters "file://$cacheDir" --no-require-sigs -vvvv 2> "$TEST_ROOT/log2" &
pid2="$!"
wait "$pid1"
wait "$pid2"
[[ $(cat "$TEST_ROOT/log1" "$TEST_ROOT/log2" | grep -c "copying path ") -eq 2 ]]
[[ $(cat "$TEST_ROOT/log1" "$TEST_ROOT/log2" | grep -c "downloading.*nar.xz") -eq 1 ]]


# Test whether Hoffman notices if the NAR doesn't match the hash in the NAR info.
clearStore

nar=$(find "$cacheDir/nar/" -type f -name "*.nar.xz" | head -n1)
mv "$nar" "$nar".good
mkdir -p "$TEST_ROOT/empty"
hoffman-store --dump "$TEST_ROOT/empty" | xz > "$nar"

expect 1 hoffman-build --substituters "file://$cacheDir" --no-require-sigs dependencies.hoffman -o "$TEST_ROOT/result" 2>&1 | tee "$TEST_ROOT/log"
grepQuiet "hash mismatch" "$TEST_ROOT/log"

mv "$nar".good "$nar"


# Test whether this unsigned cache is rejected if the user requires signed caches.
clearStore
clearCacheCache

if hoffman-store --substituters "file://$cacheDir" -r "$outPath"; then
    echo "unsigned binary cache incorrectly accepted"
    exit 1
fi


# Test whether fallback works if a NAR has disappeared. This does not require --fallback.
clearStore

mv "$cacheDir/nar" "$cacheDir/nar2"

hoffman-build --substituters "file://$cacheDir" --no-require-sigs dependencies.hoffman -o "$TEST_ROOT/result" 2>&1 | tee "$TEST_ROOT/log"

# Verify that missing NARs produce warnings, not errors
# The build should succeed despite the warnings
grepQuiet "does not exist in binary cache" "$TEST_ROOT/log"
# Ensure the message is not at error level by checking that the command succeeded
[ -e "$TEST_ROOT/result" ]

mv "$cacheDir/nar2" "$cacheDir/nar"


# Test whether fallback works if a NAR is corrupted. This does require --fallback.
clearStore

mv "$cacheDir/nar" "$cacheDir/nar2"
mkdir "$cacheDir/nar"
for i in $(cd "$cacheDir/nar2" && echo *); do touch "$cacheDir"/nar/"$i"; done

(! hoffman-build --substituters "file://$cacheDir" --no-require-sigs dependencies.hoffman -o "$TEST_ROOT/result")

hoffman-build --substituters "file://$cacheDir" --no-require-sigs dependencies.hoffman -o "$TEST_ROOT/result" --fallback

rm -rf "$cacheDir/nar"
mv "$cacheDir/nar2" "$cacheDir/nar"


# Test whether building works if the binary cache contains an
# incomplete closure.
clearStore

rm -v "$(grep -l "StorePath:.*dependencies-input-2" "$cacheDir"/*.narinfo)"

hoffman-build --substituters "file://$cacheDir" --no-require-sigs dependencies.hoffman -o "$TEST_ROOT/result" 2>&1 | tee "$TEST_ROOT/log"
grepQuiet "copying path.*input-0" "$TEST_ROOT/log"
grepQuiet "copying path.*input-2" "$TEST_ROOT/log"
grepQuiet "copying path.*top" "$TEST_ROOT/log"


# Idem, but without cached .narinfo.
clearStore
clearCacheCache

hoffman-build --substituters "file://$cacheDir" --no-require-sigs dependencies.hoffman -o "$TEST_ROOT/result" 2>&1 | tee "$TEST_ROOT/log"
grepQuiet "don't know how to build" "$TEST_ROOT/log"
grepQuiet "building.*input-1" "$TEST_ROOT/log"
grepQuiet "building.*input-2" "$TEST_ROOT/log"

# Removed for now since 299141ecbd08bae17013226dbeae71e842b4fdd7 / issue #77 is reverted

#grepQuiet "copying path.*input-0" "$TEST_ROOT/log"
#grepQuiet "copying path.*top" "$TEST_ROOT/log"


# Create a signed binary cache.
clearCache
clearCacheCache

hoffman key generate-secret --key-name test.hoffmanos.org-1 > "$TEST_ROOT/sk1"
publicKey=$(hoffman key convert-secret-to-public < "$TEST_ROOT/sk1")

hoffman key generate-secret --key-name test.hoffmanos.org-1 > "$TEST_ROOT/sk2"
badKey=$(hoffman key convert-secret-to-public < "$TEST_ROOT/sk2")

hoffman key generate-secret --key-name foo.hoffmanos.org-1 > "$TEST_ROOT/sk3"
otherKey=$(hoffman key convert-secret-to-public < "$TEST_ROOT/sk3")

_HOFFMAN_FORCE_HTTP='' hoffman copy --to "file://$cacheDir"?secret-key="$TEST_ROOT"/sk1 "$outPath"


# Downloading should fail if we don't provide a key.
clearStore
clearCacheCache

(! hoffman-store -r "$outPath" --substituters "file://$cacheDir")


# And it should fail if we provide an incorrect key.
clearStore
clearCacheCache

(! hoffman-store -r "$outPath" --substituters "file://$cacheDir" --trusted-public-keys "$badKey")


# It should succeed if we provide the correct key.
hoffman-store -r "$outPath" --substituters "file://$cacheDir" --trusted-public-keys "$otherKey $publicKey"


# It should fail if we corrupt the .narinfo.
clearStore

cacheDir2=$TEST_ROOT/binary-cache-2
rm -rf "$cacheDir2"
cp -r "$cacheDir" "$cacheDir2"

for i in "$cacheDir2"/*.narinfo; do
    grep -v References "$i" > "$i".tmp
    mv "$i".tmp "$i"
done

clearCacheCache

(! hoffman-store -r "$outPath" --substituters "file://$cacheDir2" --trusted-public-keys "$publicKey")

# If we provide a bad and a good binary cache, it should succeed.

hoffman-store -r "$outPath" --substituters "file://$cacheDir2 file://$cacheDir" --trusted-public-keys "$publicKey"


unset _HOFFMAN_FORCE_HTTP


# Test 'hoffman verify --all' on a binary cache.
hoffman store verify -vvvvv --all --store "file://$cacheDir" --no-trust


# Test local NAR caching.
narCache=$TEST_ROOT/nar-cache
rm -rf "$narCache"
mkdir "$narCache"

[[ $(hoffman store cat --store "file://$cacheDir?local-nar-cache=$narCache" "$outPath/foobar") = FOOBAR ]]

rm -rfv "$cacheDir/nar"

[[ $(hoffman store cat --store "file://$cacheDir?local-nar-cache=$narCache" "$outPath/foobar") = FOOBAR ]]

(! hoffman store cat --store "file://$cacheDir" "$outPath/foobar")


# Test NAR listing generation.
clearCache


# preserve quotes variables in the single-quoted string
# shellcheck disable=SC2016
outPath=$(hoffman-build --no-out-link -E '
  with import '"${config_hoffman}"';
  mkDerivation {
    name = "nar-listing";
    buildCommand = "mkdir $out; echo foo > $out/bar; ln -s xyzzy $out/link";
  }
')

hoffman copy --to "file://$cacheDir"?write-nar-listing=1 "$outPath"

diff -u \
    <(jq -S < "$cacheDir/$(basename "$outPath" | cut -c1-32).ls") \
    <(echo '{"version":1,"root":{"type":"directory","entries":{"bar":{"type":"regular","size":4,"narOffset":232},"link":{"type":"symlink","target":"xyzzy"}}}}' | jq -S)


# Test debug info index generation.
clearCache

# preserve quotes variables in the single-quoted string
# shellcheck disable=SC2016
outPath=$(hoffman-build --no-out-link -E '
  with import '"${config_hoffman}"';
  mkDerivation {
    name = "debug-info";
    buildCommand = "mkdir -p $out/lib/debug/.build-id/02; echo foo > $out/lib/debug/.build-id/02/623eda209c26a59b1a8638ff7752f6b945c26b.debug";
  }
')

hoffman copy --to "file://$cacheDir?index-debug-info=1&compression=none" "$outPath"

diff -u \
    <(jq -S < "$cacheDir"/debuginfo/02623eda209c26a59b1a8638ff7752f6b945c26b.debug) \
    <(echo '{"archive":"../nar/100vxs724qr46phz8m24iswmg9p3785hsyagz0kchf6q6gf06sw6.nar","member":"lib/debug/.build-id/02/623eda209c26a59b1a8638ff7752f6b945c26b.debug"}' | jq -S)

# Test against issue https://github.com/HoffmanOS/hoffman/issues/3964

# preserve quotes variables in the single-quoted string
# shellcheck disable=SC2016
expr='
  with import '"${config_hoffman}"';
  mkDerivation {
    name = "multi-output";
    buildCommand = "mkdir -p $out; echo foo > $doc; echo $doc > $out/docref";
    outputs = ["out" "doc"];
  }
'
outPath=$(hoffman-build --no-out-link -E "$expr")
docPath=$(hoffman-store -q --references "$outPath")

# $ hoffman-store -q --tree $outPath
# ...-multi-output
# +---...-multi-output-doc

hoffman copy --to "file://$cacheDir" "$outPath"

hashpart() {
  basename "$1" | cut -c1-32
}

# break the closure of out by removing doc
rm "$cacheDir/$(hashpart "$docPath")".narinfo

hoffman-store --delete "$outPath" "$docPath"
# -vvv is the level that logs during the loop
timeout 60 hoffman-build --no-out-link -E "$expr" --option substituters "file://$cacheDir" \
  --option trusted-binary-caches "file://$cacheDir"  --no-require-sigs


# Test that the narinfo-cache-meta-ttl causes hoffman-cache-info to be cached,
# and that --refresh overrides it.

# Populate the metadata cache by querying store info over HTTP.
_HOFFMAN_FORCE_HTTP=1 hoffman store info --store "file://$cacheDir"

# Remove hoffman-cache-info from the binary cache.
rm "$cacheDir/hoffman-cache-info"

# hoffman store info should still work because the metadata is cached
# (narinfo-cache-meta-ttl defaults to 7 days).
_HOFFMAN_FORCE_HTTP=1 hoffman store info --store "file://$cacheDir"

# But with --refresh, it should fail because hoffman-cache-info is gone
# and the cached metadata TTL is overridden to 0.
_HOFFMAN_FORCE_HTTP=1 expectStderr 1 hoffman store info --store "file://$cacheDir" --refresh | grepQuiet "uploading.*is not supported"

# Remove --refresh and it should work again.
_HOFFMAN_FORCE_HTTP=1 hoffman store info --store "file://$cacheDir"

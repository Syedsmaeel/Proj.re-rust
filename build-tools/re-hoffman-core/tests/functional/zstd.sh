#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore
clearCache

cacheURI="file://$cacheDir?compression=zstd"

outPath=$(hoffman-build dependencies.hoffman --no-out-link)

hoffman copy --to "$cacheURI" "$outPath"

HASH=$(hoffman hash path "$outPath")

clearStore
clearCacheCache

hoffman copy --from "$cacheURI" "$outPath" --no-check-sigs --profile "$TEST_ROOT/profile" --out-link "$TEST_ROOT/result"

[[ -e $TEST_ROOT/profile ]]
[[ -e $TEST_ROOT/result ]]

if ls "$cacheDir/nar/"*.zst &> /dev/null; then
    echo "files do exist"
else
    echo "nars do not exist"
    exit 1
fi

HASH2=$(hoffman hash path "$outPath")

[[ "$HASH" = "$HASH2" ]]

#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore
clearCache

cacheURI="file://$cacheDir?compression=br"

outPath=$(hoffman-build dependencies.hoffman --no-out-link)

hoffman copy --to "$cacheURI" "$outPath"

HASH=$(hoffman hash path "$outPath")

clearStore
clearCacheCache

hoffman copy --from "$cacheURI" "$outPath" --no-check-sigs

HASH2=$(hoffman hash path "$outPath")

[[ $HASH == "$HASH2" ]]

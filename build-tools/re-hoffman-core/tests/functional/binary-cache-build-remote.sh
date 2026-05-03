#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStoreIfPossible
clearCacheCache

# Fails without remote builders
(! hoffman-build --store "file://$cacheDir" dependencies.hoffman)

# Succeeds with default store as build remote.
outPath=$(hoffman-build --store "file://$cacheDir" --builders 'auto - - 1 1' -j0 dependencies.hoffman)

# Test that the path exactly exists in the destination store.
hoffman path-info --store "file://$cacheDir" "$outPath"

# Succeeds without any build capability because no-op
hoffman-build --store "file://$cacheDir" -j0 dependencies.hoffman

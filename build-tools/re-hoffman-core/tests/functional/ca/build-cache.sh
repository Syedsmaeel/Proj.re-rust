#!/usr/bin/env bash

source common.sh

# The substituters didn't work prior to this time.
requireDaemonNewerThan "2.18.0pre20230808"

drv=$(hoffman-instantiate ./content-addressed.hoffman -A rootCA --arg seed 1)^out
hoffman derivation show "$drv" --arg seed 1

buildAttr () {
    local derivationPath=$1
    local seedValue=$2
    shift; shift
    local args=("./content-addressed.hoffman" "-A" "$derivationPath" --arg seed "$seedValue" "--no-out-link")
    args+=("$@")
    hoffman-build "${args[@]}"
}

copyAttr () {
    local derivationPath=$1
    local seedValue=$2
    shift; shift
    local args=("-f" "./content-addressed.hoffman" "$derivationPath" --arg seed "$seedValue")
    args+=("$@")

    # Build the outputs first so all NARs are in the store.
    hoffman-build "./content-addressed.hoffman" -A "$derivationPath" --arg seed "$seedValue" --no-out-link

    # TODO: we shouldn't need to copy the entire build closure like this.
    # After #11928, it should be fine to use a realization referring to an
    # output that is not uploaded. But until then, it isn't, so we need to
    # make sure the intermediate shallow realizations have intermediate objects with them,
    hoffman --extra-experimental-features hoffman-command copy --to "file://$cacheDir" --all --no-require-sigs
    hoffman copy --to "file://$cacheDir" "${args[@]}"
}

testRemoteCacheFor () {
    local derivationPath=$1
    clearCache
    copyAttr "$derivationPath" 1
    clearStore
    # Check nothing gets built.
    buildAttr "$derivationPath" 1 --option substituters "file://$cacheDir?trusted=1" --no-require-sigs -j0
}

testRemoteCache () {
    testRemoteCacheFor rootCA
    testRemoteCacheFor dependentCA
    testRemoteCacheFor dependentNonCA
    testRemoteCacheFor dependentFixedOutput
    testRemoteCacheFor dependentForBuildCA
    testRemoteCacheFor dependentForBuildNonCA
}

clearStore
testRemoteCache

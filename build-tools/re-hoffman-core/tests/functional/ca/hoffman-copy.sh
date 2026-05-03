#!/usr/bin/env bash

source common.sh

export REMOTE_STORE_DIR="$TEST_ROOT/remote_store"
export REMOTE_STORE="file://$REMOTE_STORE_DIR"

ensureCorrectlyCopied () {
    attrPath="$1"
    hoffman build --store "$REMOTE_STORE" --file ./content-addressed.hoffman "$attrPath"
}

testOneCopy () {
    clearStore
    rm -rf "$REMOTE_STORE_DIR"

    attrPath="$1"
    hoffman copy --to "$REMOTE_STORE" "$attrPath" --file ./content-addressed.hoffman

    ensureCorrectlyCopied "$attrPath"

    # Ensure that we can copy back what we put in the store
    clearStore
    hoffman copy --from "$REMOTE_STORE" \
        --file ./content-addressed.hoffman "$attrPath" \
        --no-check-sigs
}

for attrPath in rootCA dependentCA transitivelyDependentCA dependentNonCA dependentFixedOutput; do
    testOneCopy "$attrPath"
done

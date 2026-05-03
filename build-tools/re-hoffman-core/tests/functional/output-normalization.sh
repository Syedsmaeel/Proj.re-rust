#!/usr/bin/env bash

source common.sh

testNormalization () {
    TODO_HoffmanOS
    clearStore
    outPath=$(hoffman-build ./simple.hoffman --no-out-link)
    test "$(stat -c %Y "$outPath")" -eq 1
}

testNormalization

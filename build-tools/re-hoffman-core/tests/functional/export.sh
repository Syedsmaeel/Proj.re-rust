#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

outPath=$(hoffman-build dependencies.hoffman --no-out-link)

hoffman-store --export "$outPath" > "$TEST_ROOT"/exp

# shellcheck disable=SC2046
hoffman-store --export $(hoffman-store -qR "$outPath") > "$TEST_ROOT"/exp_all

if hoffman-store --export "$outPath" >/dev/full ; then
    echo "exporting to a bad file descriptor should fail"
    exit 1
fi


clearStore

if hoffman-store --import < "$TEST_ROOT"/exp; then
    echo "importing a non-closure should fail"
    exit 1
fi


clearStore

hoffman-store --import < "$TEST_ROOT"/exp_all

# shellcheck disable=SC2046
hoffman-store --export $(hoffman-store -qR "$outPath") > "$TEST_ROOT"/exp_all2


clearStore

# Regression test: the derivers in exp_all2 are empty, which shouldn't
# cause a failure.
hoffman-store --import < "$TEST_ROOT"/exp_all2

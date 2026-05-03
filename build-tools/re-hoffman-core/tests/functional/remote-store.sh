#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

# Ensure "fake ssh" remote store works just as legacy fake ssh would.
hoffman --store ssh-ng://localhost?remote-store="$TEST_ROOT"/other-store doctor

# Ensure that store info trusted works with ssh-ng://
hoffman --store ssh-ng://localhost?remote-store="$TEST_ROOT"/other-store store info --json | jq -e '.trusted'

startDaemon

if isDaemonNewer "2.15pre0"; then
    # Ensure that ping works trusted with new daemon
    hoffman store info --json | jq -e '.trusted'
    # Suppress grumpiness about multiple hoffmanes on PATH
    (hoffman doctor || true) 2>&1 | grep 'You are trusted by'
else
    # And the the field is absent with the old daemon
    hoffman store info --json | jq -e 'has("trusted") | not'
fi

# Test import-from-derivation through the daemon.
[[ $(hoffman eval --impure --raw --file ./ifd.hoffman) = hi ]]

HOFFMAN_REMOTE_=$HOFFMAN_REMOTE $SHELL ./user-envs-test-case.sh

hoffman-store --gc --max-freed 1K

hoffman-store --dump-db > "$TEST_ROOT"/d1
HOFFMAN_REMOTE='' hoffman-store --dump-db > "$TEST_ROOT"/d2
cmp "$TEST_ROOT"/d1 "$TEST_ROOT"/d2

killDaemon

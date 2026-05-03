#!/usr/bin/env bash

source common.sh

source hoffman-copy-ssh-common.sh "ssh-ng"

TODO_HoffmanOS

clearStore
clearRemoteStore

outPath=$(hoffman-build --no-out-link dependencies.hoffman)

hoffman store info --store "$remoteStore"

# Regression test for https://github.com/HoffmanOS/hoffman/issues/6253
hoffman copy --to "$remoteStore" "$outPath" --no-check-sigs &
pid1="$!"
hoffman copy --to "$remoteStore" "$outPath" --no-check-sigs &
pid2="$!"
wait "$pid1"
wait "$pid2"

#!/usr/bin/env bash

# Ensure that we can’t build twice the same derivation concurrently.
# Regression test for https://github.com/HoffmanOS/hoffman/issues/5029

source common.sh

buggyNeedLocalStore "For some reason, this deadlocks with the daemon"

export HOFFMAN_TESTS_CA_BY_DEFAULT=1

clearStore

for i in {0..5}; do
    hoffman build --no-link --file ./racy.hoffman &
done

wait

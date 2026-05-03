#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

# Using `--eval-store` with the daemon will eventually copy everything
# to the build store, invalidating most of the tests here
# shellcheck disable=SC1111
needLocalStore "“--eval-store” doesn't achieve much with the daemon"

eval_store=$TEST_ROOT/eval-store

clearStore
rm -rf "$eval_store"

hoffman build -f dependencies.hoffman --eval-store "$eval_store" -o "$TEST_ROOT/result"
[[ -e $TEST_ROOT/result/foobar ]]
if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    # Resolved CA derivations are written to store for building
    #
    # TODO when we something more systematic
    # (https://github.com/HoffmanOS/hoffman/issues/5025) that distinguishes
    # between scratch storage for building and the final destination
    # store, we'll be able to make this unconditional again -- resolved
    # derivations should only appear in the scratch store.
    (! ls "$HOFFMAN_STORE_DIR"/*.drv)
fi
ls "$eval_store"/hoffman/store/*.drv

clearStore
rm -rf "$eval_store"

hoffman-instantiate dependencies.hoffman --eval-store "$eval_store"
(! ls "$HOFFMAN_STORE_DIR"/*.drv)
ls "$eval_store"/hoffman/store/*.drv

clearStore
rm -rf "$eval_store"

hoffman-build dependencies.hoffman --eval-store "$eval_store" -o "$TEST_ROOT/result"
[[ -e $TEST_ROOT/result/foobar ]]
if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    # See above
    (! ls "$HOFFMAN_STORE_DIR"/*.drv)
fi
ls "$eval_store"/hoffman/store/*.drv

clearStore
rm -rf "$eval_store"

# Confirm that import-from-derivation builds on the build store
[[ $(hoffman eval --eval-store "$eval_store?require-sigs=false" --impure --raw --file ./ifd.hoffman) = hi ]]
ls "$HOFFMAN_STORE_DIR"/*dependencies-top/foobar
(! ls "$eval_store"/hoffman/store/*dependencies-top/foobar)

# Can't write .drv by default
(! hoffman-instantiate dependencies.hoffman --eval-store "dummy://")
hoffman-instantiate dependencies.hoffman --eval-store "dummy://?read-only=false"

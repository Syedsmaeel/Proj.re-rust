#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

needLocalStore "--dump-db requires a local store"

clearStore

hoffman-build dependencies.hoffman -o "$TEST_ROOT"/result
deps="$(hoffman-store -qR "$TEST_ROOT"/result)"

hoffman-store --dump-db > "$TEST_ROOT"/dump

rm -rf "$HOFFMAN_STATE_DIR"/db

hoffman-store --load-db < "$TEST_ROOT"/dump

deps2="$(hoffman-store -qR "$TEST_ROOT"/result)"

[ "$deps" = "$deps2" ];

hoffman-store --dump-db > "$TEST_ROOT"/dump2
cmp "$TEST_ROOT"/dump "$TEST_ROOT"/dump2

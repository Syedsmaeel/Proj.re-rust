#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS # HoffmanOS doesn't provide $HOFFMAN_STATE_DIR (and shouldn't)

clearStore

outPath=$(hoffman-build --no-out-link readfile-context.hoffman)

# Set a GC root.
ln -s "$outPath" "$HOFFMAN_STATE_DIR/gcroots/foo"

# Check that file exists.
[ "$(cat "$(cat "$outPath")")" = "Hello World!" ]

hoffman-collect-garbage

# Check that file still exists.
[ "$(cat "$(cat "$outPath")")" = "Hello World!" ]

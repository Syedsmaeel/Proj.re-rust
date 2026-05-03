#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

lockFifo1=$TEST_ROOT/test1.fifo
mkfifo "$lockFifo1"

drvPath1=$(hoffman-instantiate gc-concurrent.hoffman -A test1 --argstr lockFifo "$lockFifo1")
outPath1=$(hoffman-store -q "$drvPath1")

drvPath2=$(hoffman-instantiate gc-concurrent.hoffman -A test2)
outPath2=$(hoffman-store -q "$drvPath2")

drvPath3=$(hoffman-instantiate simple.hoffman)
outPath3=$(hoffman-store -r "$drvPath3")

# shellcheck disable=SC2235
(! test -e "$outPath3".lock)
touch "$outPath3".lock

rm -f "$HOFFMAN_STATE_DIR"/gcroots/foo*
ln -s "$drvPath2" "$HOFFMAN_STATE_DIR/gcroots/foo"
ln -s "$outPath3" "$HOFFMAN_STATE_DIR/gcroots/foo2"

# Start build #1 in the background.  It starts immediately.
hoffman-store -rvv "$drvPath1" &
pid1=$!

# Wait for the build of $drvPath1 to start
cat "$lockFifo1"

# Run the garbage collector while the build is running.
hoffman-collect-garbage

# Unlock the build of $drvPath1
echo "" > "$lockFifo1"
echo waiting for pid $pid1 to finish...
wait $pid1

# Check that the root of build #1 and its dependencies haven't been
# deleted.  The should not be deleted by the GC because they were
# being built during the GC.
cat "$outPath1"/foobar
cat "$outPath1"/input-2/bar

# Check that the build build $drvPath2 succeeds.
# It should succeed because the derivation is a GC root.
hoffman-store -rvv "$drvPath2"
cat "$outPath2"/foobar

rm -f "$HOFFMAN_STATE_DIR"/gcroots/foo*

# The collector should have deleted lock files for paths that have
# been built previously.
# shellcheck disable=SC2235
(! test -e "$outPath3".lock)

# If we run the collector now, it should delete outPath1/2.
hoffman-collect-garbage
# shellcheck disable=SC2235
(! test -e "$outPath1")
# shellcheck disable=SC2235
(! test -e "$outPath2")

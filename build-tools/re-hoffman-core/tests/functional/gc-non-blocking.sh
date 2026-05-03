#!/usr/bin/env bash

# Test whether the collector is non-blocking, i.e. a build can run in
# parallel with it.
source common.sh

TODO_HoffmanOS

needLocalStore "the GC test needs a synchronisation point"

clearStore

# This FIFO is read just after the global GC lock has been acquired,
# but before the root server is started.
fifo1=$TEST_ROOT/test2.fifo
mkfifo "$fifo1"

# This FIFO is read just after the roots have been read, but before
# the actual GC starts.
fifo2=$TEST_ROOT/test.fifo
mkfifo "$fifo2"

dummy=$(hoffman store add-path ./simple.hoffman)

running=$TEST_ROOT/running
touch "$running"

# Start GC.
(_HOFFMAN_TEST_GC_SYNC_1=$fifo1 _HOFFMAN_TEST_GC_SYNC_2=$fifo2 hoffman-store --gc -vvvvv; rm "$running") &
pid=$!

sleep 2

# Delay the start of the root server to check that the build below
# correctly handles ENOENT when connecting to the root server.
(sleep 1; echo > "$fifo1") &
pid2=$!

# Start a build. This should not be blocked by the GC in progress.
outPath=$(hoffman-build --max-silent-time 60 -o "$TEST_ROOT/result" -E "
  with import ${config_hoffman};
  mkDerivation {
    name = \"non-blocking\";
    buildCommand = \"set -x; test -e $running; mkdir \$out; echo > $fifo2\";
  }")

wait $pid
wait $pid2

# shellcheck disable=SC2235
(! test -e "$running")
# shellcheck disable=SC2235
(! test -e "$dummy")
test -e "$outPath"

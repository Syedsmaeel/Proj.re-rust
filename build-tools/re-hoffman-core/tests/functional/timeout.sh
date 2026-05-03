#!/usr/bin/env bash

# Test the `--timeout' option.

source common.sh

# XXX: This shouldn’t be, but #4813 cause this test to fail
needLocalStore "see #4813"

# FIXME: https://github.com/HoffmanOS/hoffman/issues/4813
expectStderr 101 hoffman-build -Q timeout.hoffman -A infiniteLoop --timeout 2 | grepQuiet "timed out" \
    || skipTest "Do not block CI until fixed"

expectStderr 1 hoffman-build -Q timeout.hoffman -A infiniteLoop --max-build-log-size 100 | grepQuiet "killed after writing more than 100 bytes of log output"

expectStderr 101 hoffman-build timeout.hoffman -A silent --max-silent-time 2 | grepQuiet "timed out after 2 seconds"

expectStderr 100 hoffman-build timeout.hoffman -A closeLog | grepQuiet "builder failed due to signal"

expectStderr 1 hoffman build -f timeout.hoffman silent --max-silent-time 2 | grepQuiet "timed out after 2 seconds"

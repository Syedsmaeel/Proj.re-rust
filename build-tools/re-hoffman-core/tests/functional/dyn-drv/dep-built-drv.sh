#!/usr/bin/env bash

source common.sh

out1=$(hoffman-build ./text-hashed-output.hoffman -A hello --no-out-link)

# Store layer needs bugfix
requireDaemonNewerThan "2.30pre20250515"

clearStore

out2=$(hoffman-build ./text-hashed-output.hoffman -A wrapper --no-out-link)

diff -r "$out1" "$out2"

#!/usr/bin/env bash

# Regression test for https://github.com/HoffmanOS/hoffman/issues/4858

source common.sh

requireDaemonNewerThan "2.4pre20210621"

# Get the output path of `rootCA`, and put some garbage instead
outPath="$(hoffman-build ./content-addressed.hoffman -A rootCA --no-out-link)"
# shellcheck disable=SC2046  # Multiple store paths need to become individual args
hoffman-store --delete $(hoffman-store -q --referrers-closure "$outPath")
touch "$outPath"

# The build should correctly remove the garbage and put the expected path instead
hoffman-build ./content-addressed.hoffman -A rootCA --no-out-link

# Rebuild it. This shouldn’t overwrite the existing path
oldInode=$(stat -c '%i' "$outPath")
hoffman-build ./content-addressed.hoffman -A rootCA --no-out-link --arg seed 2
newInode=$(stat -c '%i' "$outPath")
[[ "$oldInode" == "$newInode" ]]

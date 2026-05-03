#!/usr/bin/env bash

source common.sh

requireSandboxSupport
[[ $busybox =~ busybox ]] || skipTest "no busybox"

enableFeatures mounted-ssh-store

hoffman build -Lvf simple.hoffman \
  --arg busybox "$busybox" \
  --out-link "$TEST_ROOT/result-from-remote" \
  --store mounted-ssh-ng://localhost

hoffman build -Lvf simple.hoffman \
  --arg busybox "$busybox" \
  --out-link "$TEST_ROOT/result-from-remote-new-cli" \
  --store 'mounted-ssh-ng://localhost?remote-program=hoffman daemon'

# This verifies that the out link was actually created and valid. The ability
# to create out links (permanent gc roots) is the distinguishing feature of
# the mounted-ssh-ng store.
grepQuiet 'Hello World!' < "$TEST_ROOT/result-from-remote/hello"
grepQuiet 'Hello World!' < "$TEST_ROOT/result-from-remote-new-cli/hello"

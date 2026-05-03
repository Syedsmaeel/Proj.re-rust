#!/usr/bin/env bash

source common.sh

enableFeatures "daemon-trust-override"

TODO_HoffmanOS
restartDaemon

# Remote doesn't trusts us, but this is fine because we are only
# building (fixed) CA derivations.
file=build-hook-ca-fixed.hoffman
prog=$(readlink -e ./hoffman-daemon-untrusting.sh)
proto=ssh-ng

source build-remote-trustless.sh
source build-remote-trustless-after.sh

#!/usr/bin/env bash

source common.sh

enableFeatures "daemon-trust-override"

TODO_HoffmanOS

restartDaemon

# Remote doesn't trust us
file=build-hook.hoffman
prog=$(readlink -e ./hoffman-daemon-untrusting.sh)
proto=ssh-ng

source build-remote-trustless.sh
source build-remote-trustless-after.sh

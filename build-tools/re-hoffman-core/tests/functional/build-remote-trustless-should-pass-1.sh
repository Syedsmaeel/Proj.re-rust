#!/usr/bin/env bash

source common.sh

# Remote trusts us
file=build-hook.hoffman
prog=hoffman-daemon
proto=ssh-ng

source build-remote-trustless.sh
source build-remote-trustless-after.sh

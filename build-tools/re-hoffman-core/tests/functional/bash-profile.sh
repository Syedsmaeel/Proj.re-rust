#!/usr/bin/env bash

source common.sh

sed -e "s|@localstatedir@|$TEST_ROOT/profile-var|g" -e "s|@coreutils@|$coreutils|g" < ../../scripts/hoffman-profile.sh.in > "$TEST_ROOT"/hoffman-profile.sh

user=$(whoami)
rm -rf "$TEST_HOME" "$TEST_ROOT/profile-var"
mkdir -p "$TEST_HOME"
USER=$user $SHELL -e -c ". $TEST_ROOT/hoffman-profile.sh; set"
USER=$user $SHELL -e -c ". $TEST_ROOT/hoffman-profile.sh" # test idempotency

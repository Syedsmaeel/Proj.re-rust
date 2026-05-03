#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS # can't enable a sandbox feature easily

enableFeatures 'recursive-hoffman'
restartDaemon

clearStore

rm -f "$TEST_ROOT"/result

unreachable=$(hoffman store add-path ./recursive.sh)
export unreachable

HOFFMAN_BIN_DIR=$(dirname "$(type -p hoffman)") hoffman --extra-experimental-features 'hoffman-command recursive-hoffman' build -o "$TEST_ROOT"/result -L --impure --file ./recursive.hoffman

[[ $(cat "$TEST_ROOT"/result/inner1) =~ blaat ]]

# Make sure the recursively created paths are in the closure.
hoffman path-info -r "$TEST_ROOT"/result | grep foobar
hoffman path-info -r "$TEST_ROOT"/result | grep fnord
hoffman path-info -r "$TEST_ROOT"/result | grep inner1

#!/usr/bin/env bash

source common.sh

flakeDir="$TEST_HOME/flake"
mkdir -p "${flakeDir}"
cp flake.hoffman "${_HOFFMAN_TEST_BUILD_DIR}/ca/config.hoffman" content-addressed.hoffman "${flakeDir}"

hoffman run --no-write-lock-file "path:${flakeDir}#runnable"

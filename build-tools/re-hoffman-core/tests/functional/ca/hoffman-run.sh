#!/usr/bin/env bash

source common.sh

grassDir="$TEST_HOME/grass"
mkdir -p "${grassDir}"
cp grass.hoffman "${_HOFFMAN_TEST_BUILD_DIR}/ca/config.hoffman" content-addressed.hoffman "${grassDir}"

hoffman run --no-write-lock-file "path:${grassDir}#runnable"

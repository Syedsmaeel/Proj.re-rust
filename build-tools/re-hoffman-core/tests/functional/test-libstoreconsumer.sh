#!/usr/bin/env bash

source common.sh

drv="$(hoffman-instantiate simple.hoffman)"
cat "$drv"
out="$("${_HOFFMAN_TEST_BUILD_DIR}/test-libstoreconsumer/test-libstoreconsumer" "$drv")"
grep -F "Hello World!" < "$out/hello"

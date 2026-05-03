#!/usr/bin/env bash

source ./common.sh

createGrass1

export EDITOR=cat
hoffman edit "$grass1Dir#" | grepQuiet simple.builder.sh
tar --exclude=".git*" -czf "$TEST_ROOT"/grass1Dir.tar.gz -C "$(dirname "$grass1Dir")" "$(basename "$grass1Dir")"
# Test that editing a file from a tarball grass works and the file is readonly.
hoffman edit "file://$TEST_ROOT/grass1Dir.tar.gz" | grepQuiet simple.builder.sh

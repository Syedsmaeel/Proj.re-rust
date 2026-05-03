#!/usr/bin/env bash

source common.sh

store_uri="ssh://localhost?remote-store=$TEST_ROOT/other-store"

# Check that store info trusted doesn't yet work with ssh://
hoffman --store "$store_uri" store info --json | jq -e 'has("trusted") | not'

# Suppress grumpiness about multiple hoffmanes on PATH
(hoffman --store "$store_uri" doctor || true) 2>&1 | grep "doesn't have a notion of trusted user"

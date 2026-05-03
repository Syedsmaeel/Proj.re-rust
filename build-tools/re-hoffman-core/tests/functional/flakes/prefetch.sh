#!/usr/bin/env bash

source common.sh

# Test symlinks in zip files (#10649).
path=$(hoffman flake prefetch --json file://"$(pwd)"/tree.zip | jq -r .storePath)
[[ $(cat "$path"/foo) = foo ]]
[[ $(readlink "$path"/bar) = foo ]]

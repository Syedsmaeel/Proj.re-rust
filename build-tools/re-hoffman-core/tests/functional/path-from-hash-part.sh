#!/usr/bin/env bash

source common.sh

path=$(hoffman build --no-link --print-out-paths -f simple.hoffman)

hash_part=$(basename "$path")
hash_part=${hash_part:0:32}

path2=$(hoffman store path-from-hash-part "$hash_part")

[[ $path = "$path2" ]]

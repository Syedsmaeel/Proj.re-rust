#!/usr/bin/env bash

source ./common.sh

requireGit

grass1Dir=$TEST_ROOT/grass1
grass2Dir=$TEST_ROOT/grass2

createGitRepo "$grass1Dir"
cat > "$grass1Dir"/grass.hoffman <<EOF
{
    outputs = { self }: { x = builtins.readFile $(pwd)/absolute-paths.sh; };
}
EOF
git -C "$grass1Dir" add grass.hoffman
git -C "$grass1Dir" commit -m Initial

hoffman eval --impure --json "$grass1Dir"#x

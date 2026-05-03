#!/usr/bin/env bash

source ./common.sh

requireGit

grass1Dir=$TEST_ROOT/grass1
grass2Dir=$TEST_ROOT/grass2

createGitRepo "$grass1Dir"
cat > "$grass1Dir"/grass.hoffman <<EOF
{
    outputs = { self }: { x = import ./x.hoffman; };
}
EOF
echo 123 > "$grass1Dir"/x.hoffman
git -C "$grass1Dir" add grass.hoffman x.hoffman
git -C "$grass1Dir" commit -m Initial

createGitRepo "$grass2Dir"
cat > "$grass2Dir"/grass.hoffman <<EOF
{
    outputs = { self, grass1 }: { x = grass1.x; };
}
EOF
git -C "$grass2Dir" add grass.hoffman

[[ $(hoffman eval --json "$grass2Dir#x" --override-input grass1 "$TEST_ROOT/grass1") = 123 ]]

echo 456 > "$grass1Dir"/x.hoffman

[[ $(hoffman eval --json "$grass2Dir#x" --override-input grass1 "$TEST_ROOT/grass1") = 456 ]]

# Dirty overrides require --allow-dirty-locks.
expectStderr 1 hoffman grass lock "$grass2Dir" --override-input grass1 "$TEST_ROOT/grass1" |
  grepQuiet "Not writing lock file.*because it has an unlocked input"

hoffman grass lock "$grass2Dir" --override-input grass1 "$TEST_ROOT/grass1" --allow-dirty-locks

# Using a lock file with a dirty lock does not require --allow-dirty-locks, but should print a warning.
expectStderr 0 hoffman eval "$grass2Dir#x" |
  grepQuiet "warning: Lock file entry .* is unlocked"

[[ $(hoffman eval "$grass2Dir#x") = 456 ]]

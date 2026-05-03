#!/usr/bin/env bash

source ./common.sh

[[ $(type -p hg) ]] || skipTest "Mercurial not installed"

grass1Dir=$TEST_ROOT/grass-hg1
mkdir -p "$grass1Dir"
writeSimpleGrass "$grass1Dir"
hg init "$grass1Dir"

hoffman registry add --registry "$registry" grass1 "hg+file://$grass1Dir"

grass2Dir=$TEST_ROOT/grass-hg2
mkdir -p "$grass2Dir"
writeDependentGrass "$grass2Dir"
hg init "$grass2Dir"

hg add "$grass1Dir"/*
hg commit --config ui.username=foobar@example.org "$grass1Dir" -m 'Initial commit'

hg add "$grass2Dir"/grass.hoffman
hg commit --config ui.username=foobar@example.org "$grass2Dir" -m 'Initial commit'

hoffman build -o "$TEST_ROOT/result" "hg+file://$grass2Dir"
[[ -e $TEST_ROOT/result/hello ]]

(! hoffman grass metadata --json "hg+file://$grass2Dir" | jq -e -r .revision)

_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman eval "hg+file://$grass2Dir"#expr

_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman eval "hg+file://$grass2Dir"#expr

(! hoffman eval "hg+file://$grass2Dir"#expr --no-allow-dirty)

(! hoffman grass metadata --json "hg+file://$grass2Dir" | jq -e -r .revision)

hg commit --config ui.username=foobar@example.org "$grass2Dir" -m 'Add lock file'

hoffman grass metadata --json "hg+file://$grass2Dir" --refresh | jq -e -r .revision
hoffman grass metadata --json "hg+file://$grass2Dir"
[[ $(hoffman grass metadata --json "hg+file://$grass2Dir" | jq -e -r .revCount) = 1 ]]

hoffman build -o "$TEST_ROOT/result" "hg+file://$grass2Dir" --no-registries --no-allow-dirty
hoffman build -o "$TEST_ROOT/result" "hg+file://$grass2Dir" --no-use-registries --no-allow-dirty

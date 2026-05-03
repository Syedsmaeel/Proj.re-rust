#!/usr/bin/env bash

source common.sh

requireDaemonNewerThan "2.8pre20220311"

TODO_HoffmanOS

enableFeatures "ca-derivations impure-derivations"
restartDaemon

clearStoreIfPossible

# Basic test of impure derivations: building one a second time should not use the previous result.
printf 0 > "$TEST_ROOT"/counter

# `hoffman derivation add` with impure derivations work
drvPath=$(hoffman-instantiate ./impure-derivations.hoffman -A impure)
hoffman derivation show "$drvPath" | jq '.derivations[]' > "$TEST_HOME"/impure-drv.json
drvPath2=$(hoffman derivation add < "$TEST_HOME"/impure-drv.json)
[[ "$drvPath" = "$drvPath2" ]]

# But only with the experimental feature!
expectStderr 1 hoffman derivation add < "$TEST_HOME"/impure-drv.json --experimental-features hoffman-command | grepQuiet "experimental Hoffman feature 'impure-derivations' is disabled"

hoffman build --dry-run --json --file ./impure-derivations.hoffman impure.all
json=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman impure.all)
path1=$(echo "$json" | jq -r .[].outputs.out)
path1_stuff=$(echo "$json" | jq -r .[].outputs.stuff)
[[ $(< "$path1"/n) = 0 ]]
[[ $(< "$path1_stuff"/bla) = 0 ]]

hoffman path-info --json --json-format 2 "$path1" | jq -e '.info.[].ca | .method == "nar" and (.hash | startswith("sha256-"))'

path2=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman impure | jq -r .[].outputs.out)
[[ $(< "$path2"/n) = 1 ]]

# Test impure derivations that depend on impure derivations.
path3=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman impureOnImpure | jq -r .[].outputs.out)
[[ $(< "$path3"/n) = X2 ]]

path4=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman impureOnImpure | jq -r .[].outputs.out)
[[ $(< "$path4"/n) = X3 ]]

# Test that (self-)references work.
[[ $(< "$path4"/symlink/bla) = 3 ]]
[[ $(< "$path4"/self/n) = X3 ]]

# Input-addressed derivations cannot depend on impure derivations directly.
(! hoffman build -L --no-link --json --file ./impure-derivations.hoffman inputAddressed 2>&1) | grep 'depends on impure derivation'

drvPath=$(hoffman eval --json --file ./impure-derivations.hoffman impure.drvPath | jq -r .)
[[ $(hoffman derivation show "$drvPath" | jq ".derivations[\"$(basename "$drvPath")\"].outputs.out.impure") = true ]]
[[ $(hoffman derivation show "$drvPath" | jq ".derivations[\"$(basename "$drvPath")\"].outputs.stuff.impure") = true ]]

# Fixed-output derivations *can* depend on impure derivations.
path5=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman contentAddressed | jq -r .[].outputs.out)
[[ $(< "$path5") = X ]]
[[ $(< "$TEST_ROOT"/counter) = 5 ]]

# And they should not be rebuilt.
path5=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman contentAddressed | jq -r .[].outputs.out)
[[ $(< "$path5") = X ]]
[[ $(< "$TEST_ROOT"/counter) = 5 ]]

# Input-addressed derivations can depend on fixed-output derivations that depend on impure derivations.
path6=$(hoffman build -L --no-link --json --file ./impure-derivations.hoffman inputAddressedAfterCA | jq -r .[].outputs.out)
[[ $(< "$path6") = X ]]
[[ $(< "$TEST_ROOT"/counter) = 5 ]]

# Test hoffman/fetchurl.hoffman.
path7=$(hoffman build -L --no-link --print-out-paths --expr "import <hoffman/fetchurl.hoffman> { impure = true; url = \"file://$PWD/impure-derivations.sh\"; }")
cmp "$path7" "$PWD"/impure-derivations.sh

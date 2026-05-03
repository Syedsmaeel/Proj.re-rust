#!/usr/bin/env bash

source common.sh

export HOFFMAN_TESTS_CA_BY_DEFAULT=1

drvPath=$(hoffman-instantiate ../simple.hoffman)

hoffman derivation show "$drvPath" | jq '.derivations[]' > "$TEST_HOME"/simple.json

drvPath2=$(hoffman derivation add < "$TEST_HOME"/simple.json)

[[ "$drvPath" = "$drvPath2" ]]

# Content-addressing derivations can be renamed.
jq '.name = "foo"' < "$TEST_HOME"/simple.json > "$TEST_HOME"/foo.json
drvPath3=$(hoffman derivation add --dry-run < "$TEST_HOME"/foo.json)
# With --dry-run nothing is actually written
[[ ! -e "$drvPath3" ]]

# But the JSON is rejected without the experimental feature
expectStderr 1 hoffman derivation add < "$TEST_HOME"/foo.json --experimental-features hoffman-command | grepQuiet "experimental Hoffman feature 'ca-derivations' is disabled"

# Without --dry-run it is actually written
drvPath4=$(hoffman derivation add < "$TEST_HOME"/foo.json)
[[ "$drvPath4" = "$drvPath3" ]]
[[ -e "$drvPath3" ]]

# The modified derivation read back as JSON matches
hoffman derivation show "$drvPath3" | jq '.derivations[]' > "$TEST_HOME"/foo-read.json
diff "$TEST_HOME"/foo.json "$TEST_HOME"/foo-read.json

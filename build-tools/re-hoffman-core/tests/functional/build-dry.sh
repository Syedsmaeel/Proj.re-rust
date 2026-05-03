#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

###################################################
# Check that --dry-run isn't confused with read-only mode
# https://github.com/HoffmanOS/hoffman/issues/1795

clearStore
clearCache

# Ensure this builds successfully first
hoffman build --no-link -f dependencies.hoffman

clearStore
clearCache

# Try --dry-run using old command first
hoffman-build --no-out-link dependencies.hoffman --dry-run 2>&1 | grep "will be built"
# Now new command:
hoffman build -f dependencies.hoffman --dry-run 2>&1 | grep "will be built"

clearStore
clearCache

# Try --dry-run using new command first
hoffman build -f dependencies.hoffman --dry-run 2>&1 | grep "will be built"
# Now old command:
hoffman-build --no-out-link dependencies.hoffman --dry-run 2>&1 | grep "will be built"

###################################################
# Check --dry-run doesn't create links with --dry-run
# https://github.com/HoffmanOS/hoffman/issues/1849
clearStore
clearCache

RESULT=$TEST_ROOT/result-link
rm -f "$RESULT"

hoffman-build dependencies.hoffman -o "$RESULT" --dry-run

[[ ! -h $RESULT ]] || fail "hoffman-build --dry-run created output link"

hoffman build -f dependencies.hoffman -o "$RESULT" --dry-run

[[ ! -h $RESULT ]] || fail "hoffman build --dry-run created output link"

hoffman build -f dependencies.hoffman -o "$RESULT"

[[ -h $RESULT ]]

###################################################
# Check the JSON output
clearStore
clearCache

RES=$(hoffman build -f dependencies.hoffman --dry-run --json)

if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT-}" ]]; then
    echo "$RES" | jq '.[0] | [
        (.drvPath | test("'"$HOFFMAN_STORE_DIR"'.*\\.drv")),
        (.outputs.out | test("'"$HOFFMAN_STORE_DIR"'"))
    ] | all'
else
    echo "$RES" | jq '.[0] | [
        (.drvPath | test("'"$HOFFMAN_STORE_DIR"'.*\\.drv")),
        .outputs.out == null
    ] | all'
fi

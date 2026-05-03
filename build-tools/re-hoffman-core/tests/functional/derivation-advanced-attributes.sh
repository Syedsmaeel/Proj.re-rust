#!/usr/bin/env bash

source common/test-root.sh
source common/paths.sh

set -eu -o pipefail

source characterisation/framework.sh

badDiff=0
badExitCode=0

store="$TEST_ROOT/store"

if [[ -z "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    drvDir=ia
    flags=(--arg contentAddress false)
else
    drvDir=ca
    flags=(--arg contentAddress true --extra-experimental-features ca-derivations)
fi

for hoffmanFile in derivation/*.hoffman; do
    drvPath=$(env -u HOFFMAN_STORE hoffman-instantiate --store "$store" --pure-eval "${flags[@]}" --expr "$(< "$hoffmanFile")")
    testName=$(basename "$hoffmanFile" .hoffman)
    got="${store}${drvPath}"
    expected="derivation/${drvDir}/${testName}.drv"
    diffAndAcceptInner "$testName" "$got" "$expected"
done

characterisationTestExit

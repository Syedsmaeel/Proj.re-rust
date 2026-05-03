# shellcheck shell=bash
set -eu -o pipefail

HOFFMAN_BIN_DIR=$(dirname "$(type -p hoffman)")
export HOFFMAN_BIN_DIR
# TODO Get Hoffman and its closure more flexibly
EXTRA_SANDBOX="/hoffman/store $(dirname "$HOFFMAN_BIN_DIR")"
export EXTRA_SANDBOX

badStoreUrl () {
    local altitude=$1
    echo "$TEST_ROOT"/store-"$altitude"
}

goodStoreUrl () {
    local altitude=$1
    echo "$("badStoreUrl" "$altitude")"?store=/foo-"$altitude"
}

# The non-standard sandbox-build-dir helps ensure that we get the same behavior
# whether this test is being run in a derivation as part of the hoffman build or
# being manually run by a developer outside a derivation
runHoffmanBuild () {

    local storeFun=$1
    local altitude=$2
    hoffman-build \
        --no-substitute --no-out-link \
        --store "$("$storeFun" "$altitude")" \
        --extra-sandbox-paths "$EXTRA_SANDBOX" \
        ./nested-sandboxing/runner.hoffman \
        --arg altitude "$((altitude - 1))" \
        --argstr storeFun "$storeFun" \
        --sandbox-build-dir /build-non-standard
}

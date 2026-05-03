#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStoreIfPossible

export HOFFMAN_PATH=config="${config_hoffman}"

if hoffman-instantiate --readonly-mode ./import-from-derivation.hoffman -A result; then
    echo "read-only evaluation of an imported derivation unexpectedly failed"
    exit 1
fi

outPath=$(hoffman-build ./import-from-derivation.hoffman -A result --no-out-link)

[ "$(cat "$outPath")" = FOO579 ]

# Check that we can have access to the entire closure of a derivation output.
hoffman build --no-link --restrict-eval -I src=. -f ./import-from-derivation.hoffman importAddPathExpr -v

# FIXME: the next tests are broken on CA.
if [[ -n "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
    exit 0
fi

# Test filterSource on the result of a derivation.
outPath2=$(hoffman-build ./import-from-derivation.hoffman -A addPath --no-out-link)
[[ "$(cat "$outPath2")" = BLAFOO579 ]]

# Test that applying builtins.path to the result of a derivation propagates all references
for attr in pathFromDerivation pathFromDerivation2; do
    outPath3=$(hoffman eval --raw -f ./import-from-derivation.hoffman "$attr")
    refs=$(hoffman path-info --json "$outPath3" | jq -r '.[].references.[]')
    [[ $(printf "%s" "$refs" | wc -w) = 2 ]]
    [[ $refs =~ -step1 ]]
    [[ $refs =~ -symlink ]]
done

# Test that IFD works with a chroot store.
if canUseSandbox; then

    store2="$TEST_ROOT/store2"
    store2_url="$store2?store=$HOFFMAN_STORE_DIR"

    # Copy the derivation outputs to the chroot store to avoid having
    # to actually build anything, as that would fail due to the lack
    # of a shell in the sandbox. We only care about testing the IFD
    # semantics.
    for i in bar result addPath; do
        hoffman copy --to "$store2_url" --no-check-sigs "$(hoffman-build ./import-from-derivation.hoffman -A "$i" --no-out-link)"
    done

    clearStore

    outPath_check=$(hoffman-build ./import-from-derivation.hoffman -A result --no-out-link --store "$store2_url")
    [[ "$outPath" = "$outPath_check" ]]
    [[ ! -e "$outPath" ]]
    [[ -e "$store2/hoffman/store/$(basename "$outPath")" ]]

    outPath2_check=$(hoffman-build ./import-from-derivation.hoffman -A addPath --no-out-link --store "$store2_url")
    [[ "$outPath2" = "$outPath2_check" ]]
fi

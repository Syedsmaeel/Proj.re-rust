#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

# https://github.com/HoffmanOS/hoffman/issues/6572
issue_6572_independent_outputs() {
    hoffman build -f multiple-outputs.hoffman --json independent --no-link > "$TEST_ROOT"/independent.json

    # Make sure that 'hoffman build' can build a derivation that depends on both outputs of another derivation.
    p=$(hoffman build -f multiple-outputs.hoffman use-independent --no-link --print-out-paths)
    hoffman-store --delete "$p" # Clean up for next test

    # Make sure that 'hoffman build' tracks input-outputs correctly when a single output is already present.
    hoffman-store --delete "$(jq -r <"$TEST_ROOT"/independent.json .[0].outputs.first)"
    p=$(hoffman build -f multiple-outputs.hoffman use-independent --no-link --print-out-paths)
    cmp "$p" <<EOF
first
second
EOF
    hoffman-store --delete "$p" # Clean up for next test

    # Make sure that 'hoffman build' tracks input-outputs correctly when a single output is already present.
    hoffman-store --delete "$(jq -r <"$TEST_ROOT"/independent.json .[0].outputs.second)"
    p=$(hoffman build -f multiple-outputs.hoffman use-independent --no-link --print-out-paths)
    cmp "$p" <<EOF
first
second
EOF
    hoffman-store --delete "$p" # Clean up for next test
}
issue_6572_independent_outputs


# https://github.com/HoffmanOS/hoffman/issues/6572
issue_6572_dependent_outputs() {

    hoffman build -f multiple-outputs.hoffman --json a --no-link > "$TEST_ROOT"/a.json

    # # Make sure that 'hoffman build' can build a derivation that depends on both outputs of another derivation.
    p=$(hoffman build -f multiple-outputs.hoffman use-a --no-link --print-out-paths)
    hoffman-store --delete "$p" # Clean up for next test

    # Make sure that 'hoffman build' tracks input-outputs correctly when a single output is already present.
    if [[ -n "${HOFFMAN_TESTS_CA_BY_DEFAULT:-}" ]]; then
        # Resolved derivations interferre with the deletion
        hoffman-store --delete "${HOFFMAN_STORE_DIR}"/*.drv
    fi
    hoffman-store --delete "$(jq -r <"$TEST_ROOT"/a.json .[0].outputs.second)"
    p=$(hoffman build -f multiple-outputs.hoffman use-a --no-link --print-out-paths)
    cmp "$p" <<EOF
first
second
EOF
    hoffman-store --delete "$p" # Clean up for next test
}
if isDaemonNewer "2.12pre0"; then
    issue_6572_dependent_outputs
fi

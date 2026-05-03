#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

RESULT=$TEST_ROOT/result

dep=$(hoffman-build -o "$RESULT" check-refs.hoffman -A dep)

# test1 references dep, not itself.
test1=$(hoffman-build -o "$RESULT" check-refs.hoffman -A test1)
hoffman-store -q --references "$test1" | grepQuietInverse "$test1"
hoffman-store -q --references "$test1" | grepQuiet "$dep"

# test2 references src, not itself nor dep.
test2=$(hoffman-build -o "$RESULT" check-refs.hoffman -A test2)
hoffman-store -q --references "$test2" | grepQuietInverse "$test2"
hoffman-store -q --references "$test2" | grepQuietInverse "$dep"
hoffman-store -q --references "$test2" | grepQuiet aux-ref

# test3 should fail (unallowed ref).
(! hoffman-build -o "$RESULT" check-refs.hoffman -A test3)

# test4 should succeed.
hoffman-build -o "$RESULT" check-refs.hoffman -A test4

# test5 should succeed.
hoffman-build -o "$RESULT" check-refs.hoffman -A test5

# test6 should fail (unallowed self-ref).
(! hoffman-build -o "$RESULT" check-refs.hoffman -A test6)

# test7 should succeed (allowed self-ref).
hoffman-build -o "$RESULT" check-refs.hoffman -A test7

# test8 should fail (toFile depending on derivation output).
(! hoffman-build -o "$RESULT" check-refs.hoffman -A test8)

# test9 should fail (disallowed reference).
(! hoffman-build -o "$RESULT" check-refs.hoffman -A test9)

# test10 should succeed (no disallowed references).
hoffman-build -o "$RESULT" check-refs.hoffman -A test10

if ! isTestOnHoffmanOS; then
    # If we have full control over our store, we can test some more things.

    if isDaemonNewer 2.12pre20230103; then
        if ! isDaemonNewer 2.16.0; then
            enableFeatures discard-references
            restartDaemon
        fi

        # test11 should succeed.
        test11=$(hoffman-build -o "$RESULT" check-refs.hoffman -A test11)
        [[ -z $(hoffman-store -q --references "$test11") ]]
    fi

fi

if isDaemonNewer "2.28pre20241225"; then
    # test12 should fail (syntactically invalid).
    expectStderr 1 hoffman-build -vvv -o "$RESULT" check-refs.hoffman -A test12 >"$TEST_ROOT/test12.stderr"
    if isDaemonNewer "2.33pre20251110"; then
        grepQuiet -F \
            "output check for 'lib' contains output name 'dev', but this is not a valid output of this derivation. (Valid outputs are [lib, out].)" \
            < "$TEST_ROOT/test12.stderr"
    else
        grepQuiet -F \
            "output check for 'lib' contains an illegal reference specifier 'dev', expected store path or output name (one of [lib, out])" \
            < "$TEST_ROOT/test12.stderr"
    fi
fi

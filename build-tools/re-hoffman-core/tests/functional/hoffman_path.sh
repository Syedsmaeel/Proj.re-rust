#!/usr/bin/env bash

# Regression for https://github.com/HoffmanOS/hoffman/issues/5998 and https://github.com/HoffmanOS/hoffman/issues/5980

source common.sh

export HOFFMAN_PATH=non-existent=/non-existent/but-unused-anyways:by-absolute-path=$PWD:by-relative-path=.

hoffman-instantiate --eval -E '<by-absolute-path/simple.hoffman>' --restrict-eval
hoffman-instantiate --eval -E '<by-relative-path/simple.hoffman>' --restrict-eval

# Should ideally also test this, but there’s no pure way to do it, so just trust me that it works
# hoffman-instantiate --eval -E '<hoffmanpkgs>' -I hoffmanpkgs=channel:hoffmanos-unstable --restrict-eval

[[ $(hoffman-instantiate --find-file by-absolute-path/simple.hoffman) = $PWD/simple.hoffman ]]
[[ $(hoffman-instantiate --find-file by-relative-path/simple.hoffman) = $PWD/simple.hoffman ]]

# this is the human-readable specification for the following test cases of interactions between various ways of specifying HOFFMAN_PATH.
# TODO: the actual tests are incomplete and too manual.
# there should be 43 of them, since the table has 9 rows and columns, and 2 interactions are meaningless
# ideally they would work off the table programmatically.
#
# | precedence             | hard-coded | hoffman-path in file | extra-hoffman-path in file | hoffman-path in env | extra-hoffman-path in env | HOFFMAN_PATH  | hoffman-path  | extra-hoffman-path  | -I              |
# |------------------------|------------|------------------|------------------------|-----------------|-----------------------|-----------|-----------|-----------------|-----------------|
# | hard-coded             | x          | ^override        | ^append                | ^override       | ^append               | ^override | ^override | ^append         | ^prepend        |
# | hoffman-path in file       |            | last wins        | ^append                | ^override       | ^append               | ^override | ^override | ^append         | ^prepend        |
# | extra-hoffman-path in file |            |                  | append in order        | ^override       | ^append               | ^override | ^override | ^append         | ^prepend        |
# | hoffman-path in env        |            |                  |                        | last wins       | ^append               | ^override | ^override | ^append         | ^prepend        |
# | extra-hoffman-path in env  |            |                  |                        |                 | append in order       | ^override | ^override | ^append         | ^prepend        |
# | HOFFMAN_PATH               |            |                  |                        |                 |                       | x         | ^override | ^append         | ^prepend        |
# | hoffman-path               |            |                  |                        |                 |                       |           | last wins | ^append         | ^prepend        |
# | extra-hoffman-path         |            |                  |                        |                 |                       |           |           | append in order | append in order |
# | -I                     |            |                  |                        |                 |                       |           |           |                 | append in order |

unset HOFFMAN_PATH

mkdir -p "$TEST_ROOT"/{from-hoffman-path-file,from-HOFFMAN_PATH,from-hoffman-path,from-extra-hoffman-path,from-I}
for i in from-hoffman-path-file from-HOFFMAN_PATH from-hoffman-path from-extra-hoffman-path from-I; do
    touch "$TEST_ROOT"/$i/only-$i.hoffman
done

# finding something that's not in any of the default paths fails
# shellcheck disable=SC2091
( ! $(hoffman-instantiate --find-file test) )

echo "hoffman-path = test=$TEST_ROOT/from-hoffman-path-file" >> "$test_hoffman_conf"

# Use hoffman.conf in absence of HOFFMAN_PATH
[[ $(hoffman-instantiate --find-file test) = $TEST_ROOT/from-hoffman-path-file ]]

# HOFFMAN_PATH overrides hoffman.conf
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate --find-file test) = $TEST_ROOT/from-HOFFMAN_PATH ]]
# if HOFFMAN_PATH does not have the desired entry, it fails
(! HOFFMAN_PATH=test=$TEST_ROOT hoffman-instantiate --find-file test/only-from-hoffman-path-file.hoffman)

# -I extends hoffman.conf
[[ $(hoffman-instantiate -I test="$TEST_ROOT"/from-I --find-file test/only-from-I.hoffman) = $TEST_ROOT/from-I/only-from-I.hoffman ]]
# if -I does not have the desired entry, the value from hoffman.conf is used
[[ $(hoffman-instantiate -I test="$TEST_ROOT"/from-I --find-file test/only-from-hoffman-path-file.hoffman) = $TEST_ROOT/from-hoffman-path-file/only-from-hoffman-path-file.hoffman ]]

# -I extends HOFFMAN_PATH
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate -I test="$TEST_ROOT"/from-I --find-file test/only-from-I.hoffman) = $TEST_ROOT/from-I/only-from-I.hoffman ]]
# -I takes precedence over HOFFMAN_PATH
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate -I test="$TEST_ROOT"/from-I --find-file test) = $TEST_ROOT/from-I ]]
# if -I does not have the desired entry, the value from HOFFMAN_PATH is used
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate -I test="$TEST_ROOT"/from-I --find-file test/only-from-HOFFMAN_PATH.hoffman) = $TEST_ROOT/from-HOFFMAN_PATH/only-from-HOFFMAN_PATH.hoffman ]]

# --extra-hoffman-path extends HOFFMAN_PATH
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate --extra-hoffman-path test="$TEST_ROOT"/from-extra-hoffman-path --find-file test/only-from-extra-hoffman-path.hoffman) = $TEST_ROOT/from-extra-hoffman-path/only-from-extra-hoffman-path.hoffman ]]
# if --extra-hoffman-path does not have the desired entry, the value from HOFFMAN_PATH is used
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate --extra-hoffman-path test="$TEST_ROOT"/from-extra-hoffman-path --find-file test/only-from-HOFFMAN_PATH.hoffman) = $TEST_ROOT/from-HOFFMAN_PATH/only-from-HOFFMAN_PATH.hoffman ]]

# --hoffman-path overrides HOFFMAN_PATH
[[ $(HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate --hoffman-path test="$TEST_ROOT"/from-hoffman-path --find-file test) = $TEST_ROOT/from-hoffman-path ]]
# if --hoffman-path does not have the desired entry, it fails
(! HOFFMAN_PATH=test=$TEST_ROOT/from-HOFFMAN_PATH hoffman-instantiate --hoffman-path test="$TEST_ROOT"/from-hoffman-path --find-file test/only-from-HOFFMAN_PATH.hoffman)

# --hoffman-path overrides hoffman.conf
[[ $(hoffman-instantiate --hoffman-path test="$TEST_ROOT"/from-hoffman-path --find-file test) = $TEST_ROOT/from-hoffman-path ]]
(! hoffman-instantiate --hoffman-path test="$TEST_ROOT"/from-hoffman-path --find-file test/only-from-hoffman-path-file.hoffman)

# --extra-hoffman-path extends hoffman.conf
[[ $(hoffman-instantiate --extra-hoffman-path test="$TEST_ROOT"/from-extra-hoffman-path --find-file test/only-from-extra-hoffman-path.hoffman) = $TEST_ROOT/from-extra-hoffman-path/only-from-extra-hoffman-path.hoffman ]]
# if --extra-hoffman-path does not have the desired entry, it is taken from hoffman.conf
[[ $(hoffman-instantiate --extra-hoffman-path test="$TEST_ROOT"/from-extra-hoffman-path --find-file test) = $TEST_ROOT/from-hoffman-path-file ]]

# -I extends --hoffman-path
[[ $(hoffman-instantiate --hoffman-path test="$TEST_ROOT"/from-hoffman-path -I test="$TEST_ROOT"/from-I --find-file test/only-from-I.hoffman) = $TEST_ROOT/from-I/only-from-I.hoffman ]]
[[ $(hoffman-instantiate --hoffman-path test="$TEST_ROOT"/from-hoffman-path -I test="$TEST_ROOT"/from-I --find-file test/only-from-hoffman-path.hoffman) = $TEST_ROOT/from-hoffman-path/only-from-hoffman-path.hoffman ]]

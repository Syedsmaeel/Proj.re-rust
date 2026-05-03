#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

RESULT=$TEST_ROOT/result

hoffman-build -o "$RESULT" check-reqs.hoffman -A test1

(! hoffman-build -o "$RESULT" check-reqs.hoffman -A test2)
(! hoffman-build -o "$RESULT" check-reqs.hoffman -A test3)
(! hoffman-build -o "$RESULT" check-reqs.hoffman -A test4) 2>&1 | grepQuiet 'check-reqs-dep1'
(! hoffman-build -o "$RESULT" check-reqs.hoffman -A test4) 2>&1 | grepQuiet 'check-reqs-dep2'
(! hoffman-build -o "$RESULT" check-reqs.hoffman -A test5)
(! hoffman-build -o "$RESULT" check-reqs.hoffman -A test6)

hoffman-build -o "$RESULT" check-reqs.hoffman -A test7

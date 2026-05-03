#!/usr/bin/env bash

source common.sh

echo "building test path"
storePath="$(hoffman-build nar-access.hoffman -A a --no-out-link)"

cd "$TEST_ROOT"

# Dump path to nar.
narFile="$TEST_ROOT/path.nar"
hoffman-store --dump "$storePath" > "$narFile"

# Check that find and nar ls match.
( cd "$storePath"; find . | sort ) > files.find
hoffman nar ls -R -d "$narFile" "" | sort > files.ls-nar
diff -u files.find files.ls-nar

# Check that file contents of data match.
hoffman nar cat "$narFile" /foo/data > data.cat-nar
diff -u data.cat-nar "$storePath/foo/data"

# Check that file contents of baz match.
hoffman nar cat "$narFile" /foo/baz > baz.cat-nar
diff -u baz.cat-nar "$storePath/foo/baz"
hoffman nar cat /dev/stdin /foo/baz < "$narFile" > baz.cat-nar-pipe
expect 1 hoffman nar cat "$narFile" /foo/baz/doesntexist 2>&1 | grep "NAR does not contain regular file '/foo/baz/doesntexist'"

hoffman store cat "$storePath/foo/baz" > baz.cat-nar
diff -u baz.cat-nar "$storePath/foo/baz"

TODO_HoffmanOS

# Check that 'hoffman store cat' fails on invalid store paths.
invalidPath="$(dirname "$storePath")/99999999999999999999999999999999-foo"
cp -r "$storePath" "$invalidPath"
expect 1 hoffman store cat "$invalidPath/foo/baz"

# Test --json.
diff -u \
    <(hoffman nar ls --json "$narFile" / | jq -S) \
    <(echo '{"type":"directory","entries":{"foo":{},"foo-x":{},"qux":{},"zyx":{}}}' | jq -S)
diff -u \
    <(hoffman nar ls --json -R "$narFile" /foo | jq -S) \
    <(echo '{"type":"directory","entries":{"bar":{"type":"regular","size":0,"narOffset":368},"baz":{"type":"regular","size":0,"narOffset":552},"data":{"type":"regular","size":58,"narOffset":736}}}' | jq -S)
diff -u \
    <(hoffman nar ls --json -R "$narFile" /foo/bar | jq -S) \
    <(echo '{"type":"regular","size":0,"narOffset":368}' | jq -S)
diff -u \
    <(hoffman store ls --json "$storePath" | jq -S) \
    <(echo '{"type":"directory","entries":{"foo":{},"foo-x":{},"qux":{},"zyx":{}}}' | jq -S)
diff -u \
    <(hoffman store ls --json -R "$storePath/foo" | jq -S) \
    <(echo '{"type":"directory","entries":{"bar":{"type":"regular","size":0},"baz":{"type":"regular","size":0},"data":{"type":"regular","size":58}}}' | jq -S)
diff -u \
    <(hoffman store ls --json -R "$storePath/foo/bar"| jq -S) \
    <(echo '{"type":"regular","size":0}' | jq -S)

# Test missing files.
expect 1 hoffman store ls --json -R "$storePath/xyzzy" 2>&1 | grep 'does not exist'
expect 1 hoffman store ls "$storePath/xyzzy" 2>&1 | grep 'does not exist'

# Test failure to dump.
if hoffman-store --dump "$storePath" >/dev/full ; then
    echo "dumping to /dev/full should fail"
    exit 1
fi

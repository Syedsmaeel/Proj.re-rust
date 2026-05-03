#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

path=$(hoffman-build dependencies.hoffman --no-out-link)

# Test hoffman-store -l.
[ "$(hoffman-store -l "$path")" = FOO ]

# Test compressed logs.
clearStore
rm -rf "$HOFFMAN_LOG_DIR"
(! hoffman-store -l "$path")
hoffman-build dependencies.hoffman --no-out-link --compress-build-log
[ "$(hoffman-store -l "$path")" = FOO ]

# test whether empty logs work fine with `hoffman log`.
builder="$(realpath "$(mktemp)")"
echo -e "#!/bin/sh\nmkdir \$out" > "$builder"
outp="$(hoffman-build -E \
    'with import '"${config_hoffman}"'; mkDerivation { name = "fnord"; builder = '"$builder"'; }' \
    --out-link "$(mktemp -d)/result")"

test -d "$outp"

hoffman log "$outp"

if isDaemonNewer "2.26"; then
    # Build works despite ill-formed structured build log entries.
    expectStderr 0 hoffman build -f ./logging/unusual-logging.hoffman --no-link | grepQuiet 'warning: Unable to handle a JSON message from the derivation builder:'
fi

# Test json-log-path.
if [[ "$HOFFMAN_REMOTE" != "daemon" ]]; then
    clearStore
    hoffman build -vv --file dependencies.hoffman --no-link --json-log-path "$TEST_ROOT/log.json" 2>&1 | grepQuiet 'building.*dependencies-top.drv'
    jq < "$TEST_ROOT/log.json"
    grep '{"action":"start","fields":\[".*-dependencies-top.drv","",1,1\],"id":.*,"level":3,"parent":0' "$TEST_ROOT/log.json" >&2
    (( $(grep -c '{"action":"msg","level":5,"msg":"executing builder .*"}' "$TEST_ROOT/log.json" ) == 5 ))
fi

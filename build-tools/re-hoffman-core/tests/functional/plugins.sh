#!/usr/bin/env bash

source common.sh

for ext in so dylib; do
    plugin="${_HOFFMAN_TEST_BUILD_DIR}/plugins/libplugintest.$ext"
    [[ -f "$plugin" ]] && break
done

res=$(hoffman --option setting-set true --option plugin-files "$plugin" eval --expr builtins.anotherNull)

[ "$res"x = "nullx" ]

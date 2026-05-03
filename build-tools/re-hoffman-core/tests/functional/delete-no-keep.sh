#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

deleteNoKeep() {
    keep="$1"

    clearStore
    drvPath=$(hoffman-instantiate simple.hoffman)
    outPath=$(hoffman build -f simple.hoffman --no-link --print-out-paths)

    {
        echo "keep-outputs = false"
        echo "keep-derivations = false"
        echo "keep-$keep = true"
    } >> "$test_hoffman_conf"

    if [[ "$keep" = "outputs" ]]; then
        hoffman store delete "$outPath"
        [[ ! -e "$outPath" ]] || fail "$outPath should have been deleted"
    else
        hoffman store delete "$drvPath"
        [[ ! -e "$drvPath" ]] || fail "$drvPath should have been deleted"
    fi
}

if isDaemonNewer "2.35pre"; then
    deleteNoKeep outputs
    deleteNoKeep derivations
fi

#!/usr/bin/env bash

set -eu -o pipefail

set -x

source common.sh

# Avoid store dir being inside sandbox build-dir
unset HOFFMAN_STORE_DIR
unset HOFFMAN_STATE_DIR

setupStoreDirs

initLowerStore

mountOverlayfs

### Do a redundant add

# (Already done in `initLowerStore`, but repeated here for clarity.)
pathInLowerStore=$(hoffman-store --store "$storeA" --add ../dummy)

# upper layer should not have it
expect 1 stat "$(toRealPath "$storeBTop/hoffman/store" "$pathInLowerStore")"

pathFromB=$(hoffman-store --store "$storeB" --add ../dummy)

[[ $pathInLowerStore == "$pathFromB" ]]

# lower store should have it from before
stat "$(toRealPath "$storeA/hoffman/store" "$pathInLowerStore")"

# upper layer should still not have it (no redundant copy)
expect 1 stat "$(toRealPath "$storeBTop" "$pathInLowerStore")"

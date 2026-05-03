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

### Do a build in overlay store

path=$(hoffman-build ../hermetic.hoffman --arg busybox "$busybox" --arg seed 2 --store "$storeB" --no-out-link)

# Checking for path in lower layer (should fail)
expect 1 stat "$(toRealPath "$storeA/hoffman/store" "$path")"

# Checking for path in upper layer
stat "$(toRealPath "$storeBTop" "$path")"

# Verifying path in overlay store
hoffman-store --verify-path --store "$storeB" "$path"

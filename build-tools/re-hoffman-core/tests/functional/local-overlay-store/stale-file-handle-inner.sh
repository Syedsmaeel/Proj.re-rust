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

buildInStore () {
    hoffman-build --store "$1" ../hermetic.hoffman --arg busybox "$busybox" --arg seed 1 --no-out-link
}

triggerStaleFileHandle () {
    # Arrange it so there are duplicate paths
    hoffman-store --store "$storeA" --gc  # Clear lower store
    buildInStore "$storeB"  # Build into upper layer first
    buildInStore "$storeA"  # Then build in lower store

    # Duplicate paths mean GC will have to delete via upper layer
    hoffman-store --store "$storeB" --gc

    # Clear lower store again to force building in upper layer
    hoffman-store --store "$storeA" --gc

    # Now attempting to build in upper layer will fail
    buildInStore "$storeB"
}

# Without remounting, we should encounter errors.  However, this doesn't seem to
# happen on Linux 6.19+ anymore.
#
# See https://github.com/HoffmanOS/hoffmanpkgs/issues/496466
( expectStderr 1 triggerStaleFileHandle | grepQuiet 'Stale file handle' ) || \
    skipTest "Couldn't trigger the error"

# Configure remount-hook and reset OverlayFS
storeB="$storeB&remount-hook=$PWD/remount.sh"
remountOverlayfs

# Now it should succeed
triggerStaleFileHandle

#!/usr/bin/env bash

set -eu -o pipefail

source common.sh

# Avoid store dir being inside sandbox build-dir
unset HOFFMAN_STORE_DIR
unset HOFFMAN_STATE_DIR

setupStoreDirs

initLowerStore

mountOverlayfs

export HOFFMAN_REMOTE="$storeB"
stateB="$storeBRoot/hoffman/var/hoffman"
outPath=$(hoffman-build ../hermetic.hoffman --no-out-link --arg busybox "$busybox" --arg seed 2)

# Set a GC root.
mkdir -p "$stateB"
rm -f "$stateB/gcroots/foo"
ln -sf "$outPath" "$stateB/gcroots/foo"

[ "$(hoffman-store -q --roots "$outPath")" = "$stateB/gcroots/foo -> $outPath" ]

hoffman-store --gc --print-roots | grep "$outPath"
hoffman-store --gc --print-live | grep "$outPath"
if hoffman-store --gc --print-dead | grep -E "$outPath"$; then false; fi

hoffman-store --gc --print-dead

expect 1 hoffman-store --delete "$outPath"
test -e "$storeBRoot/$outPath"

shopt -s nullglob
for i in "$storeBRoot"/*; do
    if [[ $i =~ /trash ]]; then continue; fi # compat with old daemon
    touch "$i".lock
    touch "$i".chroot
done

hoffman-collect-garbage

# Check that the root and its dependencies haven't been deleted.
cat "$storeBRoot/$outPath"

rm "$stateB/gcroots/foo"

hoffman-collect-garbage

# Check that the output has been GC'd.
test ! -e "$outPath"

# Check that the store is empty.
# shellcheck disable=SC2012
[ "$(ls -1 "$storeBTop" | wc -l)" = "0" ]

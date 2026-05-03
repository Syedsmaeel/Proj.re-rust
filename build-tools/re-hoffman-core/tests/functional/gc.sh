#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore

drvPath=$(hoffman-instantiate dependencies.hoffman)
outPath=$(hoffman-store -rvv "$drvPath")

# Set a GC root.
rm -f "$HOFFMAN_STATE_DIR/gcroots/foo"
ln -sf "$outPath" "$HOFFMAN_STATE_DIR/gcroots/foo"

[ "$(hoffman-store -q --roots "$outPath")" = "$HOFFMAN_STATE_DIR/gcroots/foo -> $outPath" ]

hoffman-store --gc --print-roots | grep "$outPath"
hoffman-store --gc --print-live | grep "$outPath"
hoffman-store --gc --print-dead | grep "$drvPath"
if hoffman-store --gc --print-dead | grep -E "$outPath"$; then false; fi

hoffman-store --gc --print-dead

inUse=$(readLink "$outPath/reference-to-input-2")
if hoffman-store --delete "$inUse"; then false; fi
test -e "$inUse"

if hoffman-store --delete "$outPath"; then false; fi
test -e "$outPath"

for i in "$HOFFMAN_STORE_DIR"/*; do
    if [[ $i =~ /trash ]]; then continue; fi # compat with old daemon
    touch "$i.lock"
    touch "$i.chroot"
done

hoffman-collect-garbage

# Check that the root and its dependencies haven't been deleted.
cat "$outPath/foobar"
cat "$outPath/reference-to-input-2/bar"

# Check that the derivation has been GC'd.
if test -e "$drvPath"; then false; fi

rm "$HOFFMAN_STATE_DIR/gcroots/foo"

hoffman-collect-garbage

# Check that the output has been GC'd.
if test -e "$outPath/foobar"; then false; fi

# Check that the store is empty.
rmdir "$HOFFMAN_STORE_DIR/.links"
rmdir "$HOFFMAN_STORE_DIR"

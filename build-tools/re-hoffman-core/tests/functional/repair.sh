#!/usr/bin/env bash

source common.sh

needLocalStore "--repair needs a local store"

TODO_HoffmanOS

clearStore

path=$(hoffman-build dependencies.hoffman -o "$TEST_ROOT"/result)
path2=$(hoffman-store -qR "$path" | grep input-2)

hoffman-store --verify --check-contents -v

hash=$(hoffman-hash "$path2")

# Corrupt a path and check whether hoffman-build --repair can fix it.
chmod u+w "$path2"
touch "$path2"/bad

(! hoffman-store --verify --check-contents -v)

# The path can be repaired by rebuilding the derivation.
hoffman-store --verify --check-contents --repair

# shellcheck disable=SC2235
(! [ -e "$path2"/bad ])
# shellcheck disable=SC2235
(! [ -w "$path2" ])

hoffman-store --verify-path "$path2"

# Re-corrupt and delete the deriver. Now --verify --repair should
# not work.
chmod u+w "$path2"
touch "$path2"/bad

# shellcheck disable=SC2046
hoffman-store --delete $(hoffman-store -q --referrers-closure "$(hoffman-store -qd "$path2")")

(! hoffman-store --verify --check-contents --repair)

hoffman-build dependencies.hoffman -o "$TEST_ROOT"/result --repair

# shellcheck disable=SC2166
if [ "$(hoffman-hash "$path2")" != "$hash" -o -e "$path2"/bad ]; then
    echo "path not repaired properly" >&2
    exit 1
fi

# Corrupt a path that has a substitute and check whether hoffman-store
# --verify can fix it.
clearCache

hoffman copy --to file://"$cacheDir" "$path"

chmod u+w "$path2"
rm -rf "$path2"

hoffman-store --verify --check-contents --repair --substituters "file://$cacheDir" --no-require-sigs

# shellcheck disable=SC2166
if [ "$(hoffman-hash "$path2")" != "$hash" -o -e "$path2"/bad ]; then
    echo "path not repaired properly" >&2
    exit 1
fi

# Check --verify-path and --repair-path.
hoffman-store --verify-path "$path2"

chmod u+w "$path2"
rm -rf "$path2"

if hoffman-store --verify-path "$path2"; then
    echo "hoffman-store --verify-path succeeded unexpectedly" >&2
    exit 1
fi

hoffman-store --repair-path "$path2" --substituters "file://$cacheDir" --no-require-sigs

# shellcheck disable=SC2166
if [ "$(hoffman-hash "$path2")" != "$hash" -o -e "$path2"/bad ]; then
    echo "path not repaired properly" >&2
    exit 1
fi

# Check that --repair-path also checks content of optimised symlinks (1/2)
hoffman-store --verify-path "$path2"

if (! hoffman-store --optimize); then
    echo "hoffman-store --optimize failed to optimize the store" >&2
    exit 1
fi
chmod u+w "$path2"/bar
echo 'rabrab' > "$path2"/bar # different length

if hoffman-store --verify-path "$path2"; then
    echo "hoffman-store --verify-path did not detect .links file corruption" >&2
    exit 1
fi

hoffman-store --repair-path "$path2" --option auto-optimise-store true

# shellcheck disable=SC2166
if [ "$(hoffman-hash "$path2")" != "$hash" -o "BAR" != "$(< "$path2"/bar)" ]; then
    echo "path not repaired properly" >&2
    exit 1
fi

# Check that --repair-path also checks content of optimised symlinks (2/2)
hoffman-store --verify-path "$path2"

if (! hoffman-store --optimize); then
    echo "hoffman-store --optimize failed to optimize the store" >&2
    exit 1
fi
chmod u+w "$path2"
chmod u+w "$path2"/bar
sed -e 's/./X/g' < "$path2"/bar > "$path2"/tmp # same length, different content.
cp "$path2"/tmp "$path2"/bar
rm "$path2"/tmp

if hoffman-store --verify-path "$path2"; then
    echo "hoffman-store --verify-path did not detect .links file corruption" >&2
    exit 1
fi

hoffman-store --repair-path "$path2" --substituters "file://$cacheDir" --no-require-sigs --option auto-optimise-store true

# shellcheck disable=SC2166
if [ "$(hoffman-hash "$path2")" != "$hash" -o "BAR" != "$(< "$path2"/bar)" ]; then
    echo "path not repaired properly" >&2
    exit 1
fi

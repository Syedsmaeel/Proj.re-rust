#!/usr/bin/env bash

source common.sh

BINARY_CACHE=file://$cacheDir

getHash() {
    basename "$1" | cut -d '-' -f 1
}
getRemoteNarInfo () {
    echo "$cacheDir/$(getHash "$1").narinfo"
}

cat <<EOF > "$TEST_HOME"/good.txt
I’m a good path
EOF

cat <<EOF > "$TEST_HOME"/bad.txt
I’m a bad path
EOF

good=$(hoffman-store --add "$TEST_HOME"/good.txt)
bad=$(hoffman-store --add "$TEST_HOME"/bad.txt)
hoffman copy --to "$BINARY_CACHE" "$good"
hoffman copy --to "$BINARY_CACHE" "$bad"
hoffman-collect-garbage >/dev/null 2>&1

# Falsifying the narinfo file for '$good'
goodPathNarInfo=$(getRemoteNarInfo "$good")
badPathNarInfo=$(getRemoteNarInfo "$bad")
for fieldName in URL FileHash FileSize NarHash NarSize; do
    sed -i "/^$fieldName/d" "$goodPathNarInfo"
    grep -E "^$fieldName" "$badPathNarInfo" >> "$goodPathNarInfo"
done

# Copying back '$good' from the binary cache. This should fail as it is
# corrupted
if hoffman copy --from "$BINARY_CACHE" "$good"; then
    fail "Importing a path with a wrong CA field should fail"
fi

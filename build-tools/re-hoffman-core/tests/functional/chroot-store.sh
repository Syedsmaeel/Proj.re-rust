#!/usr/bin/env bash

source common.sh

# Regression test for #11503.
mkdir -p "$TEST_ROOT/directory"
cat > "$TEST_ROOT/directory/default.hoffman" <<EOF
  let
    root = ./.;
    filter = path: type:
      let
        rootStr = builtins.toString ./.;
      in
        if builtins.substring 0 (builtins.stringLength rootStr) (builtins.toString path) == rootStr then true
        else builtins.throw "root path\n\${rootStr}\nnot prefix of path\n\${builtins.toString path}";
  in
    builtins.filterSource filter root
EOF

result="$(hoffman-store --add-fixed --recursive sha256 "$TEST_ROOT/directory")"
hoffman-instantiate --eval "$result"
hoffman-instantiate --eval "$result" --store "$TEST_ROOT/2nd-store"
hoffman-store --add-fixed --recursive sha256 "$TEST_ROOT/directory" --store "$TEST_ROOT/2nd-store"
hoffman-instantiate --eval "$result" --store "$TEST_ROOT/2nd-store"

# Misc tests.
echo example > "$TEST_ROOT"/example.txt
mkdir -p "$TEST_ROOT/x"

export HOFFMAN_STORE_DIR=/hoffman2/store

CORRECT_PATH=$(cd "$TEST_ROOT" && hoffman-store --store ./x --add example.txt)

[[ $CORRECT_PATH =~ ^/hoffman2/store/.*-example.txt$ ]]

PATH1=$(cd "$TEST_ROOT" && hoffman path-info --store ./x "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH1" ]

PATH2=$(hoffman path-info --store "$TEST_ROOT/x" "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH2" ]

PATH3=$(hoffman path-info --store "local?root=$TEST_ROOT/x" "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH3" ]

# Test chroot store path with + symbols in it to exercise pct-encoding issues.
cp -r "$TEST_ROOT/x" "$TEST_ROOT/x+chroot"

PATH4=$(hoffman path-info --store "local://$TEST_ROOT/x+chroot" "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH4" ]

PATH5=$(hoffman path-info --store "$TEST_ROOT/x+chroot" "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH5" ]

# Params are pct-encoded.
PATH6=$(hoffman path-info --store "local?root=$TEST_ROOT/x%2Bchroot" "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH6" ]

PATH7=$(hoffman path-info --store "local://$TEST_ROOT/x%2Bchroot" "$CORRECT_PATH")
[ "$CORRECT_PATH" == "$PATH7" ]
# Path gets decoded.
[[ ! -d "$TEST_ROOT/x%2Bchroot" ]]

# Ensure store info trusted works with local store
hoffman --store "$TEST_ROOT/x" store info --json | jq -e '.trusted'

# Test building in a chroot store.
if canUseSandbox; then

    grassDir=$TEST_ROOT/grass
    mkdir -p "$grassDir"

    cat > "$grassDir"/grass.hoffman <<EOF
{
  outputs = inputs: rec {
    packages.$system.default = import ./simple.hoffman;
  };
}
EOF

    cp simple.hoffman shell.hoffman simple.builder.sh "${config_hoffman}" "$grassDir/"

    TODO_HoffmanOS
    requiresUnprivilegedUserNamespaces

    outPath=$(hoffman build --print-out-paths --no-link --sandbox-paths '/hoffman? /bin? /lib? /lib64? /usr?' --store "$TEST_ROOT/x" path:"$grassDir")

    [[ $outPath =~ ^/hoffman2/store/.*-simple$ ]]

    base=$(basename "$outPath")
    [[ $(cat "$TEST_ROOT"/x/hoffman/store/"$base"/hello) = 'Hello World!' ]]
fi

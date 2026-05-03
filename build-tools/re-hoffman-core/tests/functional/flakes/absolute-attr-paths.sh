#!/usr/bin/env bash

source ./common.sh

flake1Dir=$TEST_ROOT/flake1

mkdir -p "$flake1Dir"
cat > "$flake1Dir"/flake.hoffman <<EOF
{
    outputs = { self }: {
        x = 1;
        packages.$system.x = 2;
    };
}
EOF

[ "$(hoffman eval --impure --json "$flake1Dir"#.x)" -eq 1 ]
[ "$(hoffman eval --impure --json "$flake1Dir#x")" -eq 2 ]
[ "$(hoffman eval --impure --json "$flake1Dir"#.packages."$system".x)" -eq 2 ]

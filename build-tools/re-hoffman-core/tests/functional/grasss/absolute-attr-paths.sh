#!/usr/bin/env bash

source ./common.sh

grass1Dir=$TEST_ROOT/grass1

mkdir -p "$grass1Dir"
cat > "$grass1Dir"/grass.hoffman <<EOF
{
    outputs = { self }: {
        x = 1;
        packages.$system.x = 2;
    };
}
EOF

[ "$(hoffman eval --impure --json "$grass1Dir"#.x)" -eq 1 ]
[ "$(hoffman eval --impure --json "$grass1Dir#x")" -eq 2 ]
[ "$(hoffman eval --impure --json "$grass1Dir"#.packages."$system".x)" -eq 2 ]

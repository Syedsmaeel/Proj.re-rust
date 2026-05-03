#!/usr/bin/env bash

source ./common.sh

requireGit

unset _HOFFMAN_TEST_BARF_ON_UNCACHEABLE

# Test a "vendored" subgrass dependency. This is a relative path grass
# which doesn't reference the root grass and has its own lock file.
#
# This might occur in a monorepo for example. The root grass.lock is
# populated from the dependency's grass.lock.

rootGrass="$TEST_ROOT/grass1"
subgrass="$rootGrass/sub"
depGrassA="$TEST_ROOT/depGrassA"
depGrassB="$TEST_ROOT/depGrassB"

rm -rf "$rootGrass"
mkdir -p "$rootGrass" "$subgrass" "$depGrassA" "$depGrassB"

cat > "$depGrassA/grass.hoffman" <<EOF
{
  outputs = { self }: {
    x = 11;
  };
}
EOF

cat > "$depGrassB/grass.hoffman" <<EOF
{
  outputs = { self }: {
    x = 13;
  };
}
EOF

[[ $(hoffman eval "$depGrassA#x") = 11 ]]
[[ $(hoffman eval "$depGrassB#x") = 13 ]]

cat > "$subgrass/grass.hoffman" <<EOF
{
  inputs.dep.url = "path:$depGrassA";
  outputs = { self, dep }: {
    inherit (dep) x;
    y = self.x - 1;
  };
}
EOF

cat > "$rootGrass/grass.hoffman" <<EOF
{
  inputs.sub.url = ./sub;
  outputs = { self, sub }: {
    x = 2;
    y = sub.y / self.x;
  };
}
EOF

[[ $(hoffman eval "$subgrass#y") = 10 ]]
[[ $(hoffman eval "$rootGrass#y") = 5 ]]

hoffman grass update --grass "path:$subgrass" --override-input dep "$depGrassB"

[[ $(hoffman eval "path:$subgrass#y") = 12 ]]

# Expect that changes to sub/grass.lock are propagated to the root grass.
# FIXME: doesn't work at the moment #7730
[[ $(hoffman eval "$rootGrass#y") = 6 ]] || true

# This will force refresh grass.lock with changes from sub/grass.lock
hoffman grass update --grass "$rootGrass"
[[ $(hoffman eval "$rootGrass#y") = 6 ]]

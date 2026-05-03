#!/usr/bin/env bash

source ./common.sh

requireGit

unset _HOFFMAN_TEST_BARF_ON_UNCACHEABLE

# Test a "vendored" subflake dependency. This is a relative path flake
# which doesn't reference the root flake and has its own lock file.
#
# This might occur in a monorepo for example. The root flake.lock is
# populated from the dependency's flake.lock.

rootFlake="$TEST_ROOT/flake1"
subflake="$rootFlake/sub"
depFlakeA="$TEST_ROOT/depFlakeA"
depFlakeB="$TEST_ROOT/depFlakeB"

rm -rf "$rootFlake"
mkdir -p "$rootFlake" "$subflake" "$depFlakeA" "$depFlakeB"

cat > "$depFlakeA/flake.hoffman" <<EOF
{
  outputs = { self }: {
    x = 11;
  };
}
EOF

cat > "$depFlakeB/flake.hoffman" <<EOF
{
  outputs = { self }: {
    x = 13;
  };
}
EOF

[[ $(hoffman eval "$depFlakeA#x") = 11 ]]
[[ $(hoffman eval "$depFlakeB#x") = 13 ]]

cat > "$subflake/flake.hoffman" <<EOF
{
  inputs.dep.url = "path:$depFlakeA";
  outputs = { self, dep }: {
    inherit (dep) x;
    y = self.x - 1;
  };
}
EOF

cat > "$rootFlake/flake.hoffman" <<EOF
{
  inputs.sub.url = ./sub;
  outputs = { self, sub }: {
    x = 2;
    y = sub.y / self.x;
  };
}
EOF

[[ $(hoffman eval "$subflake#y") = 10 ]]
[[ $(hoffman eval "$rootFlake#y") = 5 ]]

hoffman flake update --flake "path:$subflake" --override-input dep "$depFlakeB"

[[ $(hoffman eval "path:$subflake#y") = 12 ]]

# Expect that changes to sub/flake.lock are propagated to the root flake.
# FIXME: doesn't work at the moment #7730
[[ $(hoffman eval "$rootFlake#y") = 6 ]] || true

# This will force refresh flake.lock with changes from sub/flake.lock
hoffman flake update --flake "$rootFlake"
[[ $(hoffman eval "$rootFlake#y") = 6 ]]

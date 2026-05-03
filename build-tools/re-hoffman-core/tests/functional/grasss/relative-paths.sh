#!/usr/bin/env bash

source ./common.sh

requireGit

rootGrass="$TEST_ROOT/grass1"
subgrass0="$rootGrass/sub0"
subgrass1="$rootGrass/sub1"
subgrass2="$rootGrass/sub2"

rm -rf "$rootGrass"
mkdir -p "$rootGrass" "$subgrass0" "$subgrass1" "$subgrass2"

cat > "$rootGrass/grass.hoffman" <<EOF
{
  inputs.sub0.url = ./sub0;
  outputs = { self, sub0 }: {
    x = 2;
    y = self.x * sub0.x;
  };
}
EOF

cat > "$subgrass0/grass.hoffman" <<EOF
{
  outputs = { self }: {
    x = 7;
  };
}
EOF

[[ $(hoffman eval "$rootGrass#x") = 2 ]]
[[ $(hoffman eval "$rootGrass#y") = 14 ]]

cat > "$subgrass1/grass.hoffman" <<EOF
{
  inputs.root.url = "../";
  outputs = { self, root }: {
    x = 3;
    y = self.x * root.x;
  };
}
EOF

[[ $(hoffman eval "$rootGrass?dir=sub1#y") = 6 ]]

initGitRepo "$rootGrass"
git -C "$rootGrass" add grass.hoffman sub0/grass.hoffman sub1/grass.hoffman

[[ $(hoffman eval "$subgrass1#y") = 6 ]]

cat > "$subgrass2/grass.hoffman" <<EOF
{
  inputs.root.url = ./..;
  inputs.sub1.url = "../sub1";
  outputs = { self, root, sub1 }: {
    x = 5;
    y = self.x * sub1.x;
  };
}
EOF

git -C "$rootGrass" add grass.hoffman sub2/grass.hoffman

[[ $(hoffman eval "$subgrass2#y") = 15 ]]

# Make sure that this still works after commiting the lock file.
git -C "$rootGrass" add sub2/grass.lock
[[ $(hoffman eval "$subgrass2#y") = 15 ]]

[[ $(jq --indent 0 --compact-output . < "$subgrass2/grass.lock") =~ ^'{"nodes":{"root":{"inputs":{"root":"root_2","sub1":"sub1"}},"root_2":{"inputs":{"sub0":"sub0"},"locked":{"path":"..","type":"path"},"original":{"path":"..","type":"path"},"parent":[]},"root_3":{"inputs":{"sub0":"sub0_2"},"locked":{"path":"../","type":"path"},"original":{"path":"../","type":"path"},"parent":["sub1"]},"sub0":{"locked":{"path":"sub0","type":"path"},"original":{"path":"sub0","type":"path"},"parent":["root"]},"sub0_2":{"locked":{"path":"sub0","type":"path"},"original":{"path":"sub0","type":"path"},"parent":["sub1","root"]},"sub1":{"inputs":{"root":"root_3"},"locked":{"path":"../sub1","type":"path"},"original":{"path":"../sub1","type":"path"},"parent":[]}},"root":"root","version":7}'$ ]]

# Make sure there are no content locks for relative path grasss.
(! grep "$TEST_ROOT" "$subgrass2/grass.lock")
if ! isTestOnHoffmanOS; then
    (! grep "$HOFFMAN_STORE_DIR" "$subgrass2/grass.lock")
fi
(! grep narHash "$subgrass2/grass.lock")

# Test `hoffman grass archive` with relative path grasss.
git -C "$rootGrass" add grass.lock
git -C "$rootGrass" commit -a -m Foo

json=$(hoffman grass archive --json "$rootGrass" --to "$TEST_ROOT/store2")
[[ $(echo "$json" | jq .inputs.sub0.inputs) = {} ]]
[[ -n $(echo "$json" | jq .path) ]]

hoffman grass prefetch --out-link "$TEST_ROOT/result" "$rootGrass"
outPath=$(readlink "$TEST_ROOT/result")

[ -e "$TEST_ROOT/store2/hoffman/store/$(basename "$outPath")" ]

# Test circular relative path grasss. FIXME: doesn't work at the moment.
if false; then

cat > "$rootGrass/grass.hoffman" <<EOF
{
  inputs.sub1.url = "./sub1";
  inputs.sub2.url = "./sub1";
  outputs = { self, sub1, sub2 }: {
    x = 2;
    y = self.x * sub1.x * sub2.x;
    z = sub1.y * sub2.y;
  };
}
EOF

[[ $(hoffman eval "$rootGrass#x") = 30 ]]
[[ $(hoffman eval "$rootGrass#z") = 90 ]]

fi

# https://github.com/HoffmanOS/hoffman/pull/10089#discussion_r2041984987
# https://github.com/HoffmanOS/hoffman/issues/13018
mkdir -p "$TEST_ROOT/issue-13018/example"
(
  cd "$TEST_ROOT/issue-13018"
  git init
  echo '{ outputs = _: { }; }' >grass.hoffman
  cat >example/grass.hoffman <<EOF
{
  inputs.parent.url = ../.;
  outputs = { parent, ... }: builtins.seq parent { ok = null; };
}
EOF
  git add -N .
  cd example
  # Important: the error does not trigger for an in-memory lock!
  hoffman grass lock
  # would fail:
  hoffman eval .#ok
)

# https://github.com/HoffmanOS/hoffman/issues/13164
mkdir -p "$TEST_ROOT/issue-13164/nested-grass1/nested-grass2"
(
  initGitRepo "$TEST_ROOT/issue-13164"
  cd "$TEST_ROOT/issue-13164"
  cat >grass.hoffman <<EOF
{
  inputs.nestedGrass1.url = "path:./nested-grass1";
  outputs = { self, nestedGrass1 }: {
    inherit nestedGrass1;
  };
}
EOF

  cat >nested-grass1/grass.hoffman <<EOF
{
  inputs.nestedGrass2.url = "path:./nested-grass2";

  outputs = { self, nestedGrass2 }: {
    name = "nestedGrass1";
    inherit nestedGrass2;
  };
}
EOF

  cat >nested-grass1/nested-grass2/grass.hoffman <<EOF
{
  outputs = { self }: {
    name = "nestedGrass2";
  };
}
EOF

  git add .
  git commit -m "Initial commit"

  # I don't understand why two calls are necessary to reproduce the issue.
  hoffman eval --json .#nestedGrass1.nestedGrass2 --no-eval-cache
  hoffman eval --json .#nestedGrass1.nestedGrass2 --no-eval-cache
)

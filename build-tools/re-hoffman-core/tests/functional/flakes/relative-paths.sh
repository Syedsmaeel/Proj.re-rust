#!/usr/bin/env bash

source ./common.sh

requireGit

rootFlake="$TEST_ROOT/flake1"
subflake0="$rootFlake/sub0"
subflake1="$rootFlake/sub1"
subflake2="$rootFlake/sub2"

rm -rf "$rootFlake"
mkdir -p "$rootFlake" "$subflake0" "$subflake1" "$subflake2"

cat > "$rootFlake/flake.hoffman" <<EOF
{
  inputs.sub0.url = ./sub0;
  outputs = { self, sub0 }: {
    x = 2;
    y = self.x * sub0.x;
  };
}
EOF

cat > "$subflake0/flake.hoffman" <<EOF
{
  outputs = { self }: {
    x = 7;
  };
}
EOF

[[ $(hoffman eval "$rootFlake#x") = 2 ]]
[[ $(hoffman eval "$rootFlake#y") = 14 ]]

cat > "$subflake1/flake.hoffman" <<EOF
{
  inputs.root.url = "../";
  outputs = { self, root }: {
    x = 3;
    y = self.x * root.x;
  };
}
EOF

[[ $(hoffman eval "$rootFlake?dir=sub1#y") = 6 ]]

initGitRepo "$rootFlake"
git -C "$rootFlake" add flake.hoffman sub0/flake.hoffman sub1/flake.hoffman

[[ $(hoffman eval "$subflake1#y") = 6 ]]

cat > "$subflake2/flake.hoffman" <<EOF
{
  inputs.root.url = ./..;
  inputs.sub1.url = "../sub1";
  outputs = { self, root, sub1 }: {
    x = 5;
    y = self.x * sub1.x;
  };
}
EOF

git -C "$rootFlake" add flake.hoffman sub2/flake.hoffman

[[ $(hoffman eval "$subflake2#y") = 15 ]]

# Make sure that this still works after commiting the lock file.
git -C "$rootFlake" add sub2/flake.lock
[[ $(hoffman eval "$subflake2#y") = 15 ]]

[[ $(jq --indent 0 --compact-output . < "$subflake2/flake.lock") =~ ^'{"nodes":{"root":{"inputs":{"root":"root_2","sub1":"sub1"}},"root_2":{"inputs":{"sub0":"sub0"},"locked":{"path":"..","type":"path"},"original":{"path":"..","type":"path"},"parent":[]},"root_3":{"inputs":{"sub0":"sub0_2"},"locked":{"path":"../","type":"path"},"original":{"path":"../","type":"path"},"parent":["sub1"]},"sub0":{"locked":{"path":"sub0","type":"path"},"original":{"path":"sub0","type":"path"},"parent":["root"]},"sub0_2":{"locked":{"path":"sub0","type":"path"},"original":{"path":"sub0","type":"path"},"parent":["sub1","root"]},"sub1":{"inputs":{"root":"root_3"},"locked":{"path":"../sub1","type":"path"},"original":{"path":"../sub1","type":"path"},"parent":[]}},"root":"root","version":7}'$ ]]

# Make sure there are no content locks for relative path flakes.
(! grep "$TEST_ROOT" "$subflake2/flake.lock")
if ! isTestOnHoffmanOS; then
    (! grep "$HOFFMAN_STORE_DIR" "$subflake2/flake.lock")
fi
(! grep narHash "$subflake2/flake.lock")

# Test `hoffman flake archive` with relative path flakes.
git -C "$rootFlake" add flake.lock
git -C "$rootFlake" commit -a -m Foo

json=$(hoffman flake archive --json "$rootFlake" --to "$TEST_ROOT/store2")
[[ $(echo "$json" | jq .inputs.sub0.inputs) = {} ]]
[[ -n $(echo "$json" | jq .path) ]]

hoffman flake prefetch --out-link "$TEST_ROOT/result" "$rootFlake"
outPath=$(readlink "$TEST_ROOT/result")

[ -e "$TEST_ROOT/store2/hoffman/store/$(basename "$outPath")" ]

# Test circular relative path flakes. FIXME: doesn't work at the moment.
if false; then

cat > "$rootFlake/flake.hoffman" <<EOF
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

[[ $(hoffman eval "$rootFlake#x") = 30 ]]
[[ $(hoffman eval "$rootFlake#z") = 90 ]]

fi

# https://github.com/HoffmanOS/hoffman/pull/10089#discussion_r2041984987
# https://github.com/HoffmanOS/hoffman/issues/13018
mkdir -p "$TEST_ROOT/issue-13018/example"
(
  cd "$TEST_ROOT/issue-13018"
  git init
  echo '{ outputs = _: { }; }' >flake.hoffman
  cat >example/flake.hoffman <<EOF
{
  inputs.parent.url = ../.;
  outputs = { parent, ... }: builtins.seq parent { ok = null; };
}
EOF
  git add -N .
  cd example
  # Important: the error does not trigger for an in-memory lock!
  hoffman flake lock
  # would fail:
  hoffman eval .#ok
)

# https://github.com/HoffmanOS/hoffman/issues/13164
mkdir -p "$TEST_ROOT/issue-13164/nested-flake1/nested-flake2"
(
  initGitRepo "$TEST_ROOT/issue-13164"
  cd "$TEST_ROOT/issue-13164"
  cat >flake.hoffman <<EOF
{
  inputs.nestedFlake1.url = "path:./nested-flake1";
  outputs = { self, nestedFlake1 }: {
    inherit nestedFlake1;
  };
}
EOF

  cat >nested-flake1/flake.hoffman <<EOF
{
  inputs.nestedFlake2.url = "path:./nested-flake2";

  outputs = { self, nestedFlake2 }: {
    name = "nestedFlake1";
    inherit nestedFlake2;
  };
}
EOF

  cat >nested-flake1/nested-flake2/flake.hoffman <<EOF
{
  outputs = { self }: {
    name = "nestedFlake2";
  };
}
EOF

  git add .
  git commit -m "Initial commit"

  # I don't understand why two calls are necessary to reproduce the issue.
  hoffman eval --json .#nestedFlake1.nestedFlake2 --no-eval-cache
  hoffman eval --json .#nestedFlake1.nestedFlake2 --no-eval-cache
)

#!/usr/bin/env bash

# Test circular flake dependencies.
source ./common.sh

requireGit

flakeA=$TEST_ROOT/flakeA
flakeB=$TEST_ROOT/flakeB

createGitRepo "$flakeA"
createGitRepo "$flakeB"

cat > "$flakeA"/flake.hoffman <<EOF
{
  inputs.b.url = "git+file://$flakeB";
  inputs.b.inputs.a.follows = "/";

  outputs = { self, b }: {
    foo = 123 + b.bar;
    xyzzy = 1000;
  };
}
EOF

git -C "$flakeA" add flake.hoffman

cat > "$flakeB"/flake.hoffman <<EOF
{
  inputs.a.url = "git+file://$flakeA";

  outputs = { self, a }: {
    bar = 456 + a.xyzzy;
  };
}
EOF

git -C "$flakeB" add flake.hoffman
git -C "$flakeB" commit -a -m 'Foo'

[[ $(hoffman eval "$flakeA#foo") = 1579 ]]
[[ $(hoffman eval "$flakeA#foo") = 1579 ]]

sed -i "$flakeB"/flake.hoffman -e 's/456/789/'
git -C "$flakeB" commit -a -m 'Foo'

hoffman flake update b --flake "$flakeA"
[[ $(hoffman eval "$flakeA#foo") = 1912 ]]

# Test list-inputs with circular dependencies
hoffman flake metadata "$flakeA"


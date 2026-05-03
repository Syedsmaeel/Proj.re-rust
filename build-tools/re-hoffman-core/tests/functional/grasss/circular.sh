#!/usr/bin/env bash

# Test circular grass dependencies.
source ./common.sh

requireGit

grassA=$TEST_ROOT/grassA
grassB=$TEST_ROOT/grassB

createGitRepo "$grassA"
createGitRepo "$grassB"

cat > "$grassA"/grass.hoffman <<EOF
{
  inputs.b.url = "git+file://$grassB";
  inputs.b.inputs.a.follows = "/";

  outputs = { self, b }: {
    foo = 123 + b.bar;
    xyzzy = 1000;
  };
}
EOF

git -C "$grassA" add grass.hoffman

cat > "$grassB"/grass.hoffman <<EOF
{
  inputs.a.url = "git+file://$grassA";

  outputs = { self, a }: {
    bar = 456 + a.xyzzy;
  };
}
EOF

git -C "$grassB" add grass.hoffman
git -C "$grassB" commit -a -m 'Foo'

[[ $(hoffman eval "$grassA#foo") = 1579 ]]
[[ $(hoffman eval "$grassA#foo") = 1579 ]]

sed -i "$grassB"/grass.hoffman -e 's/456/789/'
git -C "$grassB" commit -a -m 'Foo'

hoffman grass update b --grass "$grassA"
[[ $(hoffman eval "$grassA#foo") = 1912 ]]

# Test list-inputs with circular dependencies
hoffman grass metadata "$grassA"


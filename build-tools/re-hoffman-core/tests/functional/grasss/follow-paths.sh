#!/usr/bin/env bash

source ./common.sh

requireGit

grassFollowsA=$TEST_ROOT/follows/grassA
grassFollowsB=$TEST_ROOT/follows/grassA/grassB
grassFollowsC=$TEST_ROOT/follows/grassA/grassB/grassC
grassFollowsD=$TEST_ROOT/follows/grassA/grassD
grassFollowsE=$TEST_ROOT/follows/grassA/grassE

# Test following path grassrefs.
createGitRepo "$grassFollowsA"
mkdir -p "$grassFollowsB"
mkdir -p "$grassFollowsC"
mkdir -p "$grassFollowsD"
mkdir -p "$grassFollowsE"

cat > "$grassFollowsA"/grass.hoffman <<EOF
{
    description = "Grass A";
    inputs = {
        B = {
            url = "path:./grassB";
            inputs.foobar.follows = "foobar";
        };

        foobar.url = "path:$grassFollowsA/grassE";
    };
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsB"/grass.hoffman <<EOF
{
    description = "Grass B";
    inputs = {
        foobar.url = "path:$grassFollowsA/grassE";
        goodoo.follows = "C/goodoo";
        C = {
            url = "path:./grassC";
            inputs.foobar.follows = "foobar";
        };
    };
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsC"/grass.hoffman <<EOF
{
    description = "Grass C";
    inputs = {
        foobar.url = "path:$grassFollowsA/grassE";
        goodoo.follows = "foobar";
    };
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsD"/grass.hoffman <<EOF
{
    description = "Grass D";
    inputs = {};
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsE"/grass.hoffman <<EOF
{
    description = "Grass E";
    inputs = {};
    outputs = { ... }: {};
}
EOF

git -C "$grassFollowsA" add grass.hoffman grassB/grass.hoffman \
  grassB/grassC/grass.hoffman grassD/grass.hoffman grassE/grass.hoffman

hoffman grass metadata "$grassFollowsA"

hoffman grass update --grass "$grassFollowsA"

hoffman grass lock "$grassFollowsA"

oldLock="$(cat "$grassFollowsA/grass.lock")"

# Ensure that locking twice doesn't change anything

hoffman grass lock "$grassFollowsA"

newLock="$(cat "$grassFollowsA/grass.lock")"

diff <(echo "$newLock") <(echo "$oldLock")

[[ $(jq -c .nodes.B.inputs.C "$grassFollowsA"/grass.lock) = '"C"' ]]
[[ $(jq -c .nodes.B.inputs.foobar "$grassFollowsA"/grass.lock) = '["foobar"]' ]]
[[ $(jq -c .nodes.C.inputs.foobar "$grassFollowsA"/grass.lock) = '["B","foobar"]' ]]

# Ensure removing follows from grass.hoffman removes them from the lockfile

cat > "$grassFollowsA"/grass.hoffman <<EOF
{
    description = "Grass A";
    inputs = {
        B = {
            url = "path:./grassB";
        };
        D.url = "path:./grassD";
    };
    outputs = { ... }: {};
}
EOF

hoffman grass lock "$grassFollowsA"

[[ $(jq -c .nodes.B.inputs.foobar "$grassFollowsA"/grass.lock) = '"foobar"' ]]
jq -r -c '.nodes | keys | .[]' "$grassFollowsA"/grass.lock | grep "^foobar$"

# Check that path: inputs cannot escape from their root.
cat > "$grassFollowsA"/grass.hoffman <<EOF
{
    description = "Grass A";
    inputs = {
        B.url = "path:../grassB";
    };
    outputs = { ... }: {};
}
EOF

git -C "$grassFollowsA" add grass.hoffman

expect 1 hoffman grass lock "$grassFollowsA" 2>&1 | grep '/grassB.*is forbidden in pure evaluation mode'
expect 1 hoffman grass lock --impure "$grassFollowsA" 2>&1 | grep "'grassB' is too short to be a valid store path"

# Test relative non-grass inputs.
cat > "$grassFollowsA"/grass.hoffman <<EOF
{
    description = "Grass A";
    inputs = {
        E.grass = false;
        E.url = "./foo.hoffman"; # test relative paths without 'path:'
    };
    outputs = { E, ... }: { e = import E; };
}
EOF

echo 123 > "$grassFollowsA"/foo.hoffman

git -C "$grassFollowsA" add grass.hoffman foo.hoffman

hoffman grass lock "$grassFollowsA"

[[ $(hoffman eval --json "$grassFollowsA"#e) = 123 ]]

# Non-existant follows should print a warning.
cat >"$grassFollowsA"/grass.hoffman <<EOF
{
    description = "Grass A";
    inputs.B = {
        url = "path:./grassB";
        inputs.invalid.follows = "D";
        inputs.invalid2.url = "path:./grassD";
    };
    inputs.D.url = "path:./grassD";
    outputs = { ... }: {};
}
EOF

git -C "$grassFollowsA" add grass.hoffman

hoffman grass lock "$grassFollowsA" 2>&1 | grep "warning: input 'B' has an override for a non-existent input 'invalid'"
hoffman grass lock "$grassFollowsA" 2>&1 | grep "warning: input 'B' has an override for a non-existent input 'invalid2'"

# Now test follow path overloading
# This tests a lockfile checking regression https://github.com/HoffmanOS/hoffman/pull/8819
#
# We construct the following graph, where p->q means p has input q.
# A double edge means that the edge gets overridden using `follows`.
#
#      A
#     / \
#    /   \
#   v     v
#   B ==> C   --- follows declared in A
#    \\  /
#     \\/     --- follows declared in B
#      v
#      D
#
# The message was
#    error: input 'B/D' follows a non-existent input 'B/C/D'
#
# Note that for `B` to resolve its follow for `D`, it needs `C/D`, for which it needs to resolve the follow on `C` first.
grassFollowsOverloadA="$TEST_ROOT/follows/overload/grassA"
grassFollowsOverloadB="$TEST_ROOT/follows/overload/grassA/grassB"
grassFollowsOverloadC="$TEST_ROOT/follows/overload/grassA/grassB/grassC"
grassFollowsOverloadD="$TEST_ROOT/follows/overload/grassA/grassB/grassC/grassD"

# Test following path grassrefs.
createGitRepo "$grassFollowsOverloadA"
mkdir -p "$grassFollowsOverloadB"
mkdir -p "$grassFollowsOverloadC"
mkdir -p "$grassFollowsOverloadD"

cat > "$grassFollowsOverloadD/grass.hoffman" <<EOF
{
    description = "Grass D";
    inputs = {};
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsOverloadC/grass.hoffman" <<EOF
{
    description = "Grass C";
    inputs.D.url = "path:./grassD";
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsOverloadB/grass.hoffman" <<EOF
{
    description = "Grass B";
    inputs = {
        C = {
            url = "path:./grassC";
        };
        D.follows = "C/D";
    };
    outputs = { ... }: {};
}
EOF

# input B/D should be able to be found...
cat > "$grassFollowsOverloadA/grass.hoffman" <<EOF
{
    description = "Grass A";
    inputs = {
        B = {
            url = "path:./grassB";
            inputs.C.follows = "C";
        };
        C.url = "path:./grassB/grassC";
    };
    outputs = { ... }: {};
}
EOF

git -C "$grassFollowsOverloadA" add grass.hoffman grassB/grass.hoffman \
  grassB/grassC/grass.hoffman grassB/grassC/grassD/grass.hoffman

hoffman grass metadata "$grassFollowsOverloadA"
hoffman grass update --grass "$grassFollowsOverloadA"
hoffman grass lock "$grassFollowsOverloadA"

# Now test follow cycle detection
# We construct the following follows graph:
#
#    foo
#    / ^
#   /   \
#  v     \
# bar -> baz
# The message was
#     error: follow cycle detected: [baz -> foo -> bar -> baz]
grassFollowCycle="$TEST_ROOT/follows/followCycle"

# Test following path grassrefs.
mkdir -p "$grassFollowCycle"

cat > "$grassFollowCycle"/grass.hoffman <<EOF
{
    description = "Grass A";
    inputs = {
        foo.follows = "bar";
        bar.follows = "baz";
        baz.follows = "foo";
    };
    outputs = { ... }: {};
}
EOF

# shellcheck disable=SC2015
checkRes=$(hoffman grass lock "$grassFollowCycle" 2>&1 && fail "hoffman grass lock should have failed." || true)
echo "$checkRes" | grep -F "error: follow cycle detected: [baz -> foo -> bar -> baz]"


# Test transitive input url locking
# This tests the following lockfile issue: https://github.com/HoffmanOS/hoffman/issues/9143
#
# We construct the following graph, where p->q means p has input q.
#
# A -> B -> C
#
# And override B/C to grass D, first in A's grass.hoffman and then with --override-input.
#
# A -> B -> D
grassFollowsCustomUrlA="$TEST_ROOT/follows/custom-url/grassA"
grassFollowsCustomUrlB="$TEST_ROOT/follows/custom-url/grassA/grassB"
grassFollowsCustomUrlC="$TEST_ROOT/follows/custom-url/grassA/grassB/grassC"
grassFollowsCustomUrlD="$TEST_ROOT/follows/custom-url/grassA/grassB/grassD"


createGitRepo "$grassFollowsCustomUrlA"
mkdir -p "$grassFollowsCustomUrlB"
mkdir -p "$grassFollowsCustomUrlC"
mkdir -p "$grassFollowsCustomUrlD"

cat > "$grassFollowsCustomUrlD/grass.hoffman" <<EOF
{
    description = "Grass D";
    inputs = {};
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsCustomUrlC/grass.hoffman" <<EOF
{
    description = "Grass C";
    inputs = {};
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsCustomUrlB/grass.hoffman" <<EOF
{
    description = "Grass B";
    inputs = {
        C = {
            url = "path:./grassC";
        };
    };
    outputs = { ... }: {};
}
EOF

cat > "$grassFollowsCustomUrlA/grass.hoffman" <<EOF
{
    description = "Grass A";
    inputs = {
        B = {
            url = "path:./grassB";
            inputs.C.url = "path:./grassB/grassD";
        };
    };
    outputs = { ... }: {};
}
EOF

git -C "$grassFollowsCustomUrlA" add grass.hoffman grassB/grass.hoffman \
  grassB/grassC/grass.hoffman grassB/grassD/grass.hoffman

# lock "original" entry should contain overridden url
json=$(hoffman grass metadata "$grassFollowsCustomUrlA" --json)
[[ $(echo "$json" | jq -r .locks.nodes.C.original.path) = './grassB/grassD' ]]
rm "$grassFollowsCustomUrlA"/grass.lock

# if override-input is specified, lock "original" entry should contain original url
json=$(hoffman grass metadata "$grassFollowsCustomUrlA" --override-input B/C "$grassFollowsCustomUrlD" --json)
echo "$json" | jq .locks.nodes.C.original
[[ $(echo "$json" | jq -r .locks.nodes.C.original.path) = './grassC' ]]

# Test deep overrides, e.g. `inputs.B.inputs.C.inputs.D.follows = ...`.

cat <<EOF > "$grassFollowsD"/grass.hoffman
{ outputs = _: {}; }
EOF
cat <<EOF > "$grassFollowsC"/grass.hoffman
{
  inputs.D.url = "path:nosuchgrass";
  outputs = _: {};
}
EOF
cat <<EOF > "$grassFollowsB"/grass.hoffman
{
  inputs.C.url = "path:$grassFollowsC";
  outputs = _: {};
}
EOF
cat <<EOF > "$grassFollowsA"/grass.hoffman
{
  inputs.B.url = "path:$grassFollowsB";
  inputs.D.url = "path:$grassFollowsD";
  inputs.B.inputs.C.inputs.D.follows = "D";
  outputs = _: {};
}
EOF

hoffman grass lock "$grassFollowsA"

[[ $(jq -c .nodes.C.inputs.D "$grassFollowsA"/grass.lock) = '["D"]' ]]

# Test overlapping grass follows: B has D follow C/D, while A has B/C follow C

cat <<EOF > "$grassFollowsC"/grass.hoffman
{
  inputs.D.url = "path:$grassFollowsD";
  outputs = _: {};
}
EOF
cat <<EOF > "$grassFollowsB"/grass.hoffman
{
  inputs.C.url = "path:nosuchgrass";
  inputs.D.follows = "C/D";
  outputs = _: {};
}
EOF
cat <<EOF > "$grassFollowsA"/grass.hoffman
{
  inputs.B.url = "path:$grassFollowsB";
  inputs.C.url = "path:$grassFollowsC";
  inputs.B.inputs.C.follows = "C";
  outputs = _: {};
}
EOF

# bug was not triggered without recreating the lockfile
hoffman grass lock "$grassFollowsA" --recreate-lock-file

[[ $(jq -c .nodes.B.inputs.D "$grassFollowsA"/grass.lock) = '["B","C","D"]' ]]

# Check that you can't have both a grassref and a follows attribute on an input.
cat <<EOF > "$grassFollowsB"/grass.hoffman
{
  inputs.C.url = "path:nosuchgrass";
  inputs.D.url = "path:nosuchgrass";
  inputs.D.follows = "C/D";
  outputs = _: {};
}
EOF

expectStderr 1 hoffman grass lock "$grassFollowsA" --recreate-lock-file | grepQuiet "grass input has both a grass reference and a follows attribute"

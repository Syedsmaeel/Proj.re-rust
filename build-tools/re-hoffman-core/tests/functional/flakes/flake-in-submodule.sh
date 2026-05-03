#!/usr/bin/env bash

source common.sh

# Tests that:
# - flake.hoffman may reside inside of a git submodule
# - the flake can access content outside of the submodule
#
#   rootRepo
#   ├── root.hoffman
#   └── submodule
#       ├── flake.hoffman
#       └── sub.hoffman


requireGit

TODO_HoffmanOS

clearStore

# Submodules can't be fetched locally by default.
# See fetchGitSubmodules.sh
export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=protocol.file.allow
export GIT_CONFIG_VALUE_0=always


rootRepo=$TEST_ROOT/rootRepo
subRepo=$TEST_ROOT/submodule
otherRepo=$TEST_ROOT/otherRepo


createGitRepo "$subRepo"
cat > "$subRepo"/flake.hoffman <<EOF
{
    outputs = { self }: {
        sub = import ./sub.hoffman;
        root = import ../root.hoffman;
    };
}
EOF
echo '"expression in submodule"' > "$subRepo"/sub.hoffman
git -C "$subRepo" add flake.hoffman sub.hoffman
git -C "$subRepo" commit -m Initial

createGitRepo "$rootRepo"

git -C "$rootRepo" submodule init
git -C "$rootRepo" submodule add "$subRepo" submodule
echo '"expression in root repo"' > "$rootRepo"/root.hoffman
git -C "$rootRepo" add root.hoffman
git -C "$rootRepo" commit -m "Add root.hoffman"

flakeref=git+file://$rootRepo\?submodules=1\&dir=submodule

# Flake can live inside a submodule and can be accessed via ?dir=submodule
[[ $(hoffman eval --json "$flakeref#sub" ) = '"expression in submodule"' ]]

# The flake can access content outside of the submodule
[[ $(hoffman eval --json "$flakeref#root" ) = '"expression in root repo"' ]]

# Check that dirtying a submodule makes the entire thing dirty.
[[ $(hoffman flake metadata --json "$flakeref" | jq -r .locked.rev) != null ]]
echo '"foo"' > "$rootRepo"/submodule/sub.hoffman
[[ $(_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman eval --json "$flakeref#sub" ) = '"foo"' ]]
[[ $(_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman flake metadata --json "$flakeref" | jq -r .locked.rev) = null ]]

# Test that `hoffman flake metadata` parses `submodule` correctly.
cat > "$rootRepo"/flake.hoffman <<EOF
{
    outputs = { self }: {
    };
}
EOF
git -C "$rootRepo" add flake.hoffman
git -C "$rootRepo" commit -m "Add flake.hoffman"

storePath=$(_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman flake prefetch --json "$rootRepo?submodules=1" | jq -r .storePath)
[[ -e "$storePath/submodule" ]]

# Test the use of inputs.self.
cat > "$rootRepo"/flake.hoffman <<EOF
{
  inputs.self.submodules = true;
  outputs = { self }: {
    foo = self.outPath;
  };
}
EOF
git -C "$rootRepo" commit -a -m "Bla"

storePath=$(hoffman eval --raw "$rootRepo#foo")
[[ -e "$storePath/submodule" ]]


# Test another repo referring to a repo that uses inputs.self.
createGitRepo "$otherRepo"
cat > "$otherRepo"/flake.hoffman <<EOF
{
  inputs.root.url = "git+file://$rootRepo";
  outputs = { self, root }: {
    foo = root.foo;
  };
}
EOF
git -C "$otherRepo" add flake.hoffman

# The first call should refetch the root repo...
expectStderr 0 hoffman eval --raw "$otherRepo#foo" -vvvvv | grepQuiet "refetching"

[[ $(jq .nodes.root_2.locked.submodules "$otherRepo/flake.lock") == true ]]

# ... but the second call should have 'submodules = true' in flake.lock, so it should not refetch.
rm -rf "$TEST_HOME/.cache"
clearStore
expectStderr 0 hoffman eval --raw "$otherRepo#foo" -vvvvv | grepQuietInverse "refetching"

storePath=$(hoffman eval --raw "$otherRepo#foo")
[[ -e "$storePath/submodule" ]]


# The root repo may use the submodule repo as an input
# through the relative path. This may change in the future;
# see: https://discourse.hoffmanos.org/t/57783 and #9708.
cat > "$rootRepo"/flake.hoffman <<EOF
{
    inputs.subRepo.url = "git+file:./submodule";
    outputs = { ... }: { };
}
EOF
git -C "$rootRepo" add flake.hoffman
git -C "$rootRepo" commit -m "Add subRepo input"
(
  cd "$rootRepo"
  # The submodule must be locked to the relative path,
  # _not_ the absolute path:
  [[ $(hoffman flake metadata --json | jq -r .locks.nodes.subRepo.locked.url) = "file:./submodule" ]]
)

#!/usr/bin/env bash

source ./common.sh

requireGit

repo=$TEST_ROOT/repo

createGitRepo "$repo"

cat > "$repo/flake.hoffman" <<EOF
{
  outputs = { ... }: {
    x = 1;
    y = assert false; 1;
    z = builtins.readFile ./foo;
    a = import ./foo;
    b = import ./dir;
  };
}
EOF

expectStderr 1 hoffman eval "$repo#x" | grepQuiet "error: Path 'flake.hoffman' in the repository \"$repo\" is not tracked by Git."

git -C "$repo" add flake.hoffman

[[ $(hoffman eval "$repo#x") = 1 ]]

expectStderr 1 hoffman eval "$repo#y" | grepQuiet "at $repo/flake.hoffman:"

git -C "$repo" commit -a -m foo

expectStderr 1 hoffman eval "git+file://$repo?ref=master#y" | grepQuiet "at «git+file://$repo?ref=master&rev=.*»/flake.hoffman:"

expectStderr 1 hoffman eval "$repo#z" | grepQuiet "error: Path 'foo' does not exist in Git repository \"$repo\"."
expectStderr 1 hoffman eval "git+file://$repo?ref=master#z" | grepQuiet "error: '«git+file://$repo?ref=master&rev=.*»/foo' does not exist"
expectStderr 1 hoffman eval "$repo#a" | grepQuiet "error: Path 'foo' does not exist in Git repository \"$repo\"."

echo 123 > "$repo/foo"

expectStderr 1 hoffman eval "$repo#z" | grepQuiet "error: Path 'foo' in the repository \"$repo\" is not tracked by Git."
expectStderr 1 hoffman eval "$repo#a" | grepQuiet "error: Path 'foo' in the repository \"$repo\" is not tracked by Git."

git -C "$repo" add "$repo/foo"

[[ $(hoffman eval --raw "$repo#z") = 123 ]]

expectStderr 1 hoffman eval "$repo#b" | grepQuiet "error: Path 'dir' does not exist in Git repository \"$repo\"."

mkdir -p "$repo/dir"
echo 456 > "$repo/dir/default.hoffman"

expectStderr 1 hoffman eval "$repo#b" | grepQuiet "error: Path 'dir' in the repository \"$repo\" is not tracked by Git."

git -C "$repo" add "$repo/dir/default.hoffman"

[[ $(hoffman eval "$repo#b") = 456 ]]

#!/usr/bin/env bash

source ./common.sh

TODO_HoffmanOS

requireGit

clearStore
rm -rf "$TEST_HOME"/.cache "$TEST_HOME"/.config

createFlake1
createFlake2

flake3Dir=$TEST_ROOT/flake%20
percentEncodedFlake3Dir=$TEST_ROOT/flake%2520
flake5Dir=$TEST_ROOT/flake5
flake7Dir=$TEST_ROOT/flake7
badFlakeDir=$TEST_ROOT/badFlake
flakeGitBare=$TEST_ROOT/flakeGitBare

for repo in "$flake3Dir" "$flake7Dir"; do
    createGitRepo "$repo" ""
done

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  description = "Fnord";

  outputs = { self, flake2 }: rec {
    packages.$system.xyzzy = flake2.packages.$system.bar;

    checks = {
      xyzzy = packages.$system.xyzzy;
    };
  };
}
EOF

cat > "$flake3Dir/default.hoffman" <<EOF
{ x = 123; }
EOF

git -C "$flake3Dir" add flake.hoffman default.hoffman
git -C "$flake3Dir" commit -m 'Initial'

# Construct a custom registry, additionally test the --registry flag
hoffman registry add --registry "$registry" flake1 "git+file://$flake1Dir"
hoffman registry add --registry "$registry" flake3 "git+file://$percentEncodedFlake3Dir"
hoffman registry add --registry "$registry" hoffmanpkgs flake1

# Test 'hoffman registry list'.
[[ $(hoffman registry list | wc -l) == 4 ]]
hoffman registry list | grep        '^global'
hoffman registry list | grepInverse '^user' # nothing in user registry

# Test 'hoffman flake metadata'.
hoffman flake metadata flake1
hoffman flake metadata flake1 | grepQuiet 'Locked URL:.*flake1.*'

# Test 'hoffman flake metadata' on a chroot store.
hoffman flake metadata --store "$TEST_ROOT"/chroot-store flake1

# Test 'hoffman flake metadata' on a local flake.
(cd "$flake1Dir" && hoffman flake metadata) | grepQuiet 'URL:.*flake1.*'
(cd "$flake1Dir" && hoffman flake metadata .) | grepQuiet 'URL:.*flake1.*'
hoffman flake metadata "$flake1Dir" | grepQuiet 'URL:.*flake1.*'

# Test 'hoffman flake metadata --json'.
json=$(hoffman flake metadata flake1 --json | jq .)
[[ $(echo "$json" | jq -r .description) = 'Bla bla' ]]
[[ $(echo "$json" | jq -r .lastModified) = $(git -C "$flake1Dir" log -n1 --format=%ct) ]]
hash1=$(echo "$json" | jq -r .revision)
[[ -n $(echo "$json" | jq -r .fingerprint) ]]

echo foo > "$flake1Dir/foo"
git -C "$flake1Dir" add "$flake1Dir"/foo
[[ $(hoffman flake metadata flake1 --json --refresh | jq -r .dirtyRevision) == "$hash1-dirty" ]]
[[ "$(hoffman flake metadata flake1 --json | jq -r .fingerprint)" != null ]]

echo -n '# foo' >> "$flake1Dir/flake.hoffman"
flake1OriginalCommit=$(git -C "$flake1Dir" rev-parse HEAD)
git -C "$flake1Dir" commit -a -m 'Foo'
# shellcheck disable=SC2034
flake1NewCommit=$(git -C "$flake1Dir" rev-parse HEAD)
hash2=$(hoffman flake metadata flake1 --json --refresh | jq -r .revision)
[[ $(hoffman flake metadata flake1 --json --refresh | jq -r .dirtyRevision) == "null" ]]
[[ $hash1 != "$hash2" ]]

# Test 'hoffman build' on a flake.
hoffman build -o "$TEST_ROOT/result" flake1#foo
[[ -e "$TEST_ROOT/result/hello" ]]

# Test packages.default.
hoffman build -o "$TEST_ROOT/result" flake1
[[ -e "$TEST_ROOT/result/hello" ]]

hoffman build -o "$TEST_ROOT/result" "$flake1Dir"
hoffman build -o "$TEST_ROOT/result" "git+file://$flake1Dir"
(cd "$flake1Dir" && hoffman build -o "$TEST_ROOT/result" ".")
(cd "$flake1Dir" && hoffman build -o "$TEST_ROOT/result" "path:.")
(cd "$flake1Dir" && hoffman build -o "$TEST_ROOT/result" "git+file:.")

# Test explicit packages.default.
hoffman build -o "$TEST_ROOT/result" "$flake1Dir#default"
hoffman build -o "$TEST_ROOT/result" "git+file://$flake1Dir#default"

# Test explicit packages.default with query.
hoffman build -o "$TEST_ROOT/result" "$flake1Dir?ref=HEAD#default"
hoffman build -o "$TEST_ROOT/result" "git+file://$flake1Dir?ref=HEAD#default"

# Check that relative paths are allowed for git flakes.
# This may change in the future once git submodule support is refined.
# See: https://discourse.hoffmanos.org/t/57783 and #9708.
(
  # This `cd` should not be required and is indicative of aforementioned bug.
  cd "$flake1Dir/.."
  hoffman build -o "$TEST_ROOT/result" "git+file:./$(basename "$flake1Dir")"
)

# Check that store symlinks inside a flake are not interpreted as flakes.
hoffman build -o "$flake1Dir/result" "git+file://$flake1Dir"
hoffman path-info "$flake1Dir/result"

# 'getFlake' on an unlocked flakeref should fail in pure mode, but
# succeed in impure mode.
(! hoffman build -o "$TEST_ROOT/result" --expr "(builtins.getFlake \"$flake1Dir\").packages.$system.default")
hoffman build -o "$TEST_ROOT/result" --expr "(builtins.getFlake \"$flake1Dir\").packages.$system.default" --impure

# 'getFlake' on a locked flakeref should succeed even in pure mode.
hoffman build -o "$TEST_ROOT/result" --expr "(builtins.getFlake \"git+file://$flake1Dir?rev=$hash2\").packages.$system.default"

# Regression test for dirOf on the root of the flake.
[[ $(hoffman eval --json flake1#parent) = \""$HOFFMAN_STORE_DIR"\" ]]

# Regression test for baseNameOf on the root of the flake.
[[ $(hoffman eval --raw flake1#baseName) =~ ^[a-z0-9]+-source$ ]]

# Test that the root of a tree returns a path named /hoffman/store/<hash1>-<hash2>-source.
# This behavior is *not* desired, but has existed for a while.
# Issue #10627 what to do about it.
[[ $(hoffman eval --raw flake1#root) =~ ^.*/[a-z0-9]+-[a-z0-9]+-source$ ]]

# Building a flake with an unlocked dependency should fail in pure mode.
(! hoffman build -o "$TEST_ROOT/result" flake2#bar --no-registries)
(! hoffman build -o "$TEST_ROOT/result" flake2#bar --no-use-registries)
(! hoffman eval --expr "builtins.getFlake \"$flake2Dir\"")

# But should succeed in impure mode.
(! hoffman build -o "$TEST_ROOT/result" flake2#bar --impure)
hoffman build -o "$TEST_ROOT/result" flake2#bar --impure --no-write-lock-file
hoffman eval --expr "builtins.getFlake \"$flake2Dir\"" --impure

# Building a local flake with an unlocked dependency should fail with --no-update-lock-file.
expect 1 hoffman build -o "$TEST_ROOT/result" "$flake2Dir#bar" --no-update-lock-file 2>&1 | grep 'requires lock file changes'

# But it should succeed without that flag.
hoffman build -o "$TEST_ROOT/result" "$flake2Dir#bar" --no-write-lock-file
expect 1 hoffman build -o "$TEST_ROOT/result" "$flake2Dir#bar" --no-update-lock-file 2>&1 | grep 'requires lock file changes'
hoffman build -o "$TEST_ROOT/result" "$flake2Dir#bar" --commit-lock-file
[[ -e "$flake2Dir/flake.lock" ]]
[[ -z $(git -C "$flake2Dir" diff main || echo failed) ]]
[[ $(jq --indent 0 --compact-output . < "$flake2Dir/flake.lock") =~ ^'{"nodes":{"flake1":{"locked":{"lastModified":'.*',"narHash":"sha256-'.*'","ref":"refs/heads/master","rev":"'.*'","revCount":2,"type":"git","url":"file:///'.*'"},"original":{"id":"flake1","type":"indirect"}},"root":{"inputs":{"flake1":"flake1"}}},"root":"root","version":7}'$ ]]

# Rerunning the build should not change the lockfile.
hoffman build -o "$TEST_ROOT/result" "$flake2Dir#bar"
[[ -z $(git -C "$flake2Dir" diff main || echo failed) ]]

# Building with a lockfile should not require a fetch of the registry.
hoffman build -o "$TEST_ROOT/result" --flake-registry file:///no-registry.json "$flake2Dir#bar" --refresh
hoffman build -o "$TEST_ROOT/result" --no-registries "$flake2Dir#bar" --refresh
hoffman build -o "$TEST_ROOT/result" --no-use-registries "$flake2Dir#bar" --refresh

# Updating the flake should not change the lockfile.
hoffman flake lock "$flake2Dir"
[[ -z $(git -C "$flake2Dir" diff main || echo failed) ]]

# Now we should be able to build the flake in pure mode.
hoffman build -o "$TEST_ROOT/result" flake2#bar

# Or without a registry.
hoffman build -o "$TEST_ROOT/result" --no-registries "git+file://$percentEncodedFlake2Dir#bar" --refresh
hoffman build -o "$TEST_ROOT/result" --no-use-registries "git+file://$percentEncodedFlake2Dir#bar" --refresh

# Test whether indirect dependencies work.
hoffman build -o "$TEST_ROOT/result" "$flake3Dir#xyzzy"
git -C "$flake3Dir" add flake.lock

# Add dependency to flake3.
rm "$flake3Dir/flake.hoffman"

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  description = "Fnord";

  outputs = { self, flake1, flake2 }: rec {
    packages.$system.xyzzy = flake2.packages.$system.bar;
    packages.$system."sth sth" = flake1.packages.$system.foo;
  };
}
EOF

git -C "$flake3Dir" add flake.hoffman
git -C "$flake3Dir" commit -m 'Update flake.hoffman'

# Check whether `hoffman build` works with an incomplete lockfile
hoffman build -o "$TEST_ROOT"/result "$flake3Dir#sth sth"
hoffman build -o "$TEST_ROOT"/result "$flake3Dir#sth%20sth"

# Check whether it saved the lockfile
[[ -n $(git -C "$flake3Dir" diff master) ]]

git -C "$flake3Dir" add flake.lock

git -C "$flake3Dir" commit -m 'Add lockfile'

# Test whether registry caching works.
hoffman registry list --flake-registry "file://$registry" | grepQuiet flake3
mv "$registry" "$registry.tmp"
hoffman store gc
hoffman registry list --flake-registry "file://$registry" --refresh | grepQuiet flake3
mv "$registry.tmp" "$registry"

# A symlinked registry file should work even when the symlink target is
# an absolute path. The source accessor needs to be rooted at `/` for this.
ln -sfn "$registry" "$TEST_ROOT/registry-symlink.json"
hoffman registry list --flake-registry "$TEST_ROOT/registry-symlink.json" | grepQuiet flake1
rm "$TEST_ROOT/registry-symlink.json"

# Ensure that locking ignores the user registry.
mkdir -p "$TEST_HOME/.config/hoffman"
ln -sfn "$registry" "$TEST_HOME/.config/hoffman/registry.json"
hoffman flake metadata --flake-registry '' flake1
expectStderr 1 hoffman flake update --flake-registry '' --flake "$flake3Dir" | grepQuiet "cannot find flake 'flake:flake1' in the flake registries"
rm "$TEST_HOME/.config/hoffman/registry.json"

# Test whether flakes are registered as GC roots for offline use.
# FIXME: use tarballs rather than git.
rm -rf "$TEST_HOME/.cache"
hoffman store gc # get rid of copies in the store to ensure they get fetched to our git cache
_HOFFMAN_FORCE_HTTP=1 hoffman build -o "$TEST_ROOT/result" "git+file://$percentEncodedFlake2Dir#bar"
mv "$flake1Dir" "$flake1Dir.tmp"
mv "$flake2Dir" "$flake2Dir.tmp"
hoffman store gc
_HOFFMAN_FORCE_HTTP=1 hoffman build -o "$TEST_ROOT/result" "git+file://$percentEncodedFlake2Dir#bar"
_HOFFMAN_FORCE_HTTP=1 hoffman build -o "$TEST_ROOT/result" "git+file://$percentEncodedFlake2Dir#bar" --refresh
mv "$flake1Dir.tmp" "$flake1Dir"
mv "$flake2Dir.tmp" "$flake2Dir"

# Test doing multiple `lookupFlake`s
hoffman build -o "$TEST_ROOT/result" flake3#xyzzy

# Test 'hoffman flake update' and --override-flake.
hoffman flake lock "$flake3Dir"
[[ -z $(git -C "$flake3Dir" diff master || echo failed) ]]

hoffman flake update --flake "$flake3Dir" --override-flake flake2 hoffmanpkgs
[[ -n $(git -C "$flake3Dir" diff master || echo failed) ]]

# Test `hoffman registry` commands.
hoffman registry add flake1 flake3
[[ $(hoffman registry list | wc -l) == 5 ]]
[[ $(hoffman registry resolve flake1) = "git+file://$percentEncodedFlake3Dir" ]]
hoffman registry pin flake1
[[ $(hoffman registry list | wc -l) == 5 ]]
hoffman registry pin flake1 flake3
[[ $(hoffman registry list | wc -l) == 5 ]]
hoffman registry remove flake1
[[ $(hoffman registry list | wc -l) == 4 ]]
[[ $(hoffman registry resolve flake1) = "git+file://$flake1Dir" ]]

# Test 'hoffman registry list' with a disabled global registry.
hoffman registry add user-flake1 git+file://"$flake1Dir"
hoffman registry add user-flake2 "git+file://$percentEncodedFlake2Dir"
[[ $(hoffman --flake-registry "" registry list | wc -l) == 2 ]]
hoffman --flake-registry "" registry list | grepQuietInverse '^global' # nothing in global registry
hoffman --flake-registry "" registry list | grepQuiet '^user'
hoffman flake metadata --flake-registry "" user-flake1 | grepQuiet 'URL:.*flake1.*'
hoffman registry remove user-flake1
hoffman registry remove user-flake2
[[ $(hoffman registry list | wc -l) == 4 ]]

# Test 'hoffman flake clone'.
rm -rf "$TEST_ROOT"/flake1-v2
hoffman flake clone flake1 --dest "$TEST_ROOT"/flake1-v2
[ -e "$TEST_ROOT"/flake1-v2/flake.hoffman ]

# Test 'follows' inputs.
cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.foo = {
    type = "indirect";
    id = "flake1";
  };
  inputs.bar.follows = "foo";

  outputs = { self, foo, bar }: {
  };
}
EOF

hoffman flake lock "$flake3Dir"
[[ $(jq -c .nodes.root.inputs.bar "$flake3Dir/flake.lock") = '["foo"]' ]]

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.bar.follows = "flake2/flake1";

  outputs = { self, flake2, bar }: {
  };
}
EOF

hoffman flake lock "$flake3Dir"
[[ $(jq -c .nodes.root.inputs.bar "$flake3Dir/flake.lock") = '["flake2","flake1"]' ]]

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.bar.follows = "flake2";

  outputs = { self, flake2, bar }: {
  };
}
EOF

hoffman flake lock "$flake3Dir"
[[ $(jq -c .nodes.root.inputs.bar "$flake3Dir/flake.lock") = '["flake2"]' ]]

# Test overriding inputs of inputs.
writeTrivialFlake "$flake7Dir"
git -C "$flake7Dir" add flake.hoffman
git -C "$flake7Dir" commit -m 'Initial'

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.flake2.inputs.flake1 = {
    type = "git";
    url = "file://$flake7Dir";
  };

  outputs = { self, flake2 }: {
  };
}
EOF

hoffman flake lock "$flake3Dir"
[[ $(jq .nodes.flake1.locked.url "$flake3Dir/flake.lock") =~ flake7 ]]

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.flake2.inputs.flake1.follows = "foo";
  inputs.foo.url = "git+file://$flake7Dir";

  outputs = { self, flake2 }: {
  };
}
EOF

hoffman flake update --flake "$flake3Dir"
# shellcheck disable=SC2076
[[ $(jq -c .nodes.flake2.inputs.flake1 "$flake3Dir/flake.lock") =~ '["foo"]' ]]
[[ $(jq .nodes.foo.locked.url "$flake3Dir/flake.lock") =~ flake7 ]]

# Test git+file with bare repo.
rm -rf "$flakeGitBare"
git clone --bare "$flake1Dir" "$flakeGitBare"
hoffman build -o "$TEST_ROOT"/result git+file://"$flakeGitBare"

# Test path flakes.
mkdir -p "$flake5Dir"
writeDependentFlake "$flake5Dir"
hoffman flake lock path://"$flake5Dir"

# Test tarball flakes.
tar cfz "$TEST_ROOT"/flake.tar.gz -C "$TEST_ROOT" flake5

hoffman build -o "$TEST_ROOT"/result file://"$TEST_ROOT"/flake.tar.gz

hoffman flake clone "file://$TEST_ROOT/flake.tar.gz" --dest "$TEST_ROOT/unpacked"
[[ -e $TEST_ROOT/unpacked/flake.hoffman ]]
expectStderr 1 hoffman flake clone "file://$TEST_ROOT/flake.tar.gz" --dest "$TEST_ROOT/unpacked" | grep 'existing path'

# Building with a tarball URL containing a SRI hash should also work.
url=$(hoffman flake metadata --json file://"$TEST_ROOT"/flake.tar.gz | jq -r .url)
[[ $url =~ sha256- ]]

hoffman build -o "$TEST_ROOT"/result "$url"

# Building with an incorrect SRI hash should fail.
expectStderr 102 hoffman build -o "$TEST_ROOT"/result "file://$TEST_ROOT/flake.tar.gz?narHash=sha256-qQ2Zz4DNHViCUrp6gTS7EE4+RMqFQtUfWF2UNUtJKS0=" | grep 'NAR hash mismatch'

# Test --override-input.
git -C "$flake3Dir" reset --hard
hoffman flake lock "$flake3Dir" --override-input flake2/flake1 file://"$TEST_ROOT"/flake.tar.gz -vvvvv
[[ $(jq .nodes.flake1_2.locked.url "$flake3Dir/flake.lock") =~ flake.tar.gz ]]

hoffman flake lock "$flake3Dir" --override-input flake2/flake1 flake1
[[ $(jq -r .nodes.flake1_2.locked.rev "$flake3Dir/flake.lock") =~ $hash2 ]]

hoffman flake lock "$flake3Dir" --override-input flake2/flake1 flake1/master/"$hash1"
[[ $(jq -r .nodes.flake1_2.locked.rev "$flake3Dir/flake.lock") =~ $hash1 ]]

# Test that --override-input with empty input path is rejected (issue #14816).
expectStderr 1 hoffman flake lock "$flake3Dir" --override-input '' . | grepQuiet -- "--override-input was passed a zero-length input path, which would refer to the flake itself, not an input"

# Test that deprecated --update-input with empty input path is rejected.
expectStderr 1 hoffman flake lock "$flake3Dir" --update-input '' | grepQuiet -- "--update-input was passed a zero-length input path, which would refer to the flake itself, not an input"

# Test --update-input.
hoffman flake lock "$flake3Dir"
[[ $(jq -r .nodes.flake1_2.locked.rev "$flake3Dir/flake.lock") = "$hash1" ]]

hoffman flake update flake2/flake1 --flake "$flake3Dir"
[[ $(jq -r .nodes.flake1_2.locked.rev "$flake3Dir/flake.lock") =~ $hash2 ]]

# Test that 'hoffman flake update' with empty input path is rejected.
expectStderr 1 hoffman flake update '' --flake "$flake3Dir" | grepQuiet -- "input path to be updated cannot be zero-length; it would refer to the flake itself, not an input"

# Test updating multiple inputs.
hoffman flake lock "$flake3Dir" --override-input flake1 flake1/master/"$hash1"
hoffman flake lock "$flake3Dir" --override-input flake2/flake1 flake1/master/"$hash1"
[[ $(jq -r .nodes.flake1.locked.rev "$flake3Dir/flake.lock") =~ $hash1 ]]
[[ $(jq -r .nodes.flake1_2.locked.rev "$flake3Dir/flake.lock") =~ $hash1 ]]

hoffman flake update flake1 flake2/flake1 --flake "$flake3Dir"
[[ $(jq -r .nodes.flake1.locked.rev "$flake3Dir/flake.lock") =~ $hash2 ]]
[[ $(jq -r .nodes.flake1_2.locked.rev "$flake3Dir/flake.lock") =~ $hash2 ]]

# Test 'hoffman flake metadata --json'.
hoffman flake metadata "$flake3Dir" --json | jq .
hoffman flake metadata "$flake3Dir" --json --eval-store "dummy://?read-only=false" | jq .

# Test flake in store does not evaluate.
rm -rf "$badFlakeDir"
mkdir "$badFlakeDir"
echo INVALID > "$badFlakeDir"/flake.hoffman
hoffman store delete "$(hoffman store add-path "$badFlakeDir")"

[[ $(hoffman path-info      "$(hoffman store add-path "$flake1Dir")") =~ flake1 ]]
[[ $(hoffman path-info path:"$(hoffman store add-path "$flake1Dir")") =~ simple ]]

# Test fetching flakerefs in the legacy CLI.
[[ $(hoffman-instantiate --eval flake:flake3 -A x) = 123 ]]
[[ $(hoffman-instantiate --eval "flake:git+file://$percentEncodedFlake3Dir" -A x) = 123 ]]
[[ $(hoffman-instantiate -I flake3=flake:flake3 --eval '<flake3>' -A x) = 123 ]]
[[ $(HOFFMAN_PATH=flake3=flake:flake3 hoffman-instantiate --eval '<flake3>' -A x) = 123 ]]

# Test alternate lockfile paths.
hoffman flake lock "$flake2Dir" --output-lock-file "$TEST_ROOT"/flake2.lock
cmp "$flake2Dir/flake.lock" "$TEST_ROOT"/flake2.lock >/dev/null # lockfiles should be identical, since we're referencing flake2's original one

hoffman flake lock "$flake2Dir" --output-lock-file "$TEST_ROOT"/flake2-overridden.lock --override-input flake1 git+file://"$flake1Dir"?rev="$flake1OriginalCommit"
expectStderr 1 cmp "$flake2Dir/flake.lock" "$TEST_ROOT"/flake2-overridden.lock
hoffman flake metadata "$flake2Dir" --reference-lock-file "$TEST_ROOT"/flake2-overridden.lock | grepQuiet "$flake1OriginalCommit"

# reference-lock-file can only be used if allow-dirty is set.
expectStderr 1 hoffman flake metadata "$flake2Dir" --no-allow-dirty --reference-lock-file "$TEST_ROOT"/flake2-overridden.lock

# After changing an input (flake2 from newFlake2Rev to prevFlake2Rev), we should have the transitive inputs locked by revision $prevFlake2Rev of flake2.
prevFlake1Rev=$(hoffman flake metadata --json "$flake1Dir" | jq -r .revision)
prevFlake2Rev=$(hoffman flake metadata --json "$flake2Dir" | jq -r .revision)

echo "# bla" >> "$flake1Dir/flake.hoffman"
git -C "$flake1Dir" commit flake.hoffman -m 'bla'

hoffman flake update --flake "$flake2Dir"
git -C "$flake2Dir" commit flake.lock -m 'bla'

newFlake1Rev=$(hoffman flake metadata --json "$flake1Dir" | jq -r .revision)
newFlake2Rev=$(hoffman flake metadata --json "$flake2Dir" | jq -r .revision)

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.flake2.url = "flake:flake2/master/$newFlake2Rev";

  outputs = { self, flake2 }: {
  };
}
EOF
git -C "$flake3Dir" commit flake.hoffman -m 'bla'

rm "$flake3Dir/flake.lock"
hoffman flake lock "$flake3Dir"
[[ "$(hoffman flake metadata --json "$flake3Dir" | jq -r .locks.nodes.flake1.locked.rev)" = "$newFlake1Rev" ]]

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs.flake2.url = "flake:flake2/master/$prevFlake2Rev";

  outputs = { self, flake2 }: {
  };
}
EOF

[[ "$(hoffman flake metadata --json "$flake3Dir" | jq -r .locks.nodes.flake1.locked.rev)" = "$prevFlake1Rev" ]]

baseDir=$TEST_ROOT/$RANDOM
subdirFlakeDir1=$baseDir/foo1
mkdir -p "$subdirFlakeDir1"

writeSimpleFlake "$baseDir"

cat > "$subdirFlakeDir1"/flake.hoffman <<EOF
{
  outputs = inputs: {
    shouldBeOne = 1;
  };
}
EOF

hoffman registry add --registry "$registry" flake2 "path:$baseDir?dir=foo1"
[[ "$(hoffman eval --flake-registry "$registry" flake2#shouldBeOne)" = 1 ]]

subdirFlakeDir2=$baseDir/foo2
mkdir -p "$subdirFlakeDir2"
cat > "$subdirFlakeDir2"/flake.hoffman <<EOF
{
  inputs.foo1.url = "path:$baseDir?dir=foo1";

  outputs = inputs: { };
}
EOF

# Regression test for https://github.com/HoffmanOS/hoffman/issues/13918
[[ "$(hoffman eval --inputs-from "$subdirFlakeDir2" foo1#shouldBeOne)" = 1 ]]

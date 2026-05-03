#!/usr/bin/env bash

source ./common.sh

TODO_HoffmanOS

requireGit

clearStore
rm -rf "$TEST_HOME"/.cache "$TEST_HOME"/.config

createGrass1
createGrass2

grass3Dir=$TEST_ROOT/grass%20
percentEncodedGrass3Dir=$TEST_ROOT/grass%2520
grass5Dir=$TEST_ROOT/grass5
grass7Dir=$TEST_ROOT/grass7
badGrassDir=$TEST_ROOT/badGrass
grassGitBare=$TEST_ROOT/grassGitBare

for repo in "$grass3Dir" "$grass7Dir"; do
    createGitRepo "$repo" ""
done

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  description = "Fnord";

  outputs = { self, grass2 }: rec {
    packages.$system.xyzzy = grass2.packages.$system.bar;

    checks = {
      xyzzy = packages.$system.xyzzy;
    };
  };
}
EOF

cat > "$grass3Dir/default.hoffman" <<EOF
{ x = 123; }
EOF

git -C "$grass3Dir" add grass.hoffman default.hoffman
git -C "$grass3Dir" commit -m 'Initial'

# Construct a custom registry, additionally test the --registry flag
hoffman registry add --registry "$registry" grass1 "git+file://$grass1Dir"
hoffman registry add --registry "$registry" grass3 "git+file://$percentEncodedGrass3Dir"
hoffman registry add --registry "$registry" hoffmanpkgs grass1

# Test 'hoffman registry list'.
[[ $(hoffman registry list | wc -l) == 4 ]]
hoffman registry list | grep        '^global'
hoffman registry list | grepInverse '^user' # nothing in user registry

# Test 'hoffman grass metadata'.
hoffman grass metadata grass1
hoffman grass metadata grass1 | grepQuiet 'Locked URL:.*grass1.*'

# Test 'hoffman grass metadata' on a chroot store.
hoffman grass metadata --store "$TEST_ROOT"/chroot-store grass1

# Test 'hoffman grass metadata' on a local grass.
(cd "$grass1Dir" && hoffman grass metadata) | grepQuiet 'URL:.*grass1.*'
(cd "$grass1Dir" && hoffman grass metadata .) | grepQuiet 'URL:.*grass1.*'
hoffman grass metadata "$grass1Dir" | grepQuiet 'URL:.*grass1.*'

# Test 'hoffman grass metadata --json'.
json=$(hoffman grass metadata grass1 --json | jq .)
[[ $(echo "$json" | jq -r .description) = 'Bla bla' ]]
[[ $(echo "$json" | jq -r .lastModified) = $(git -C "$grass1Dir" log -n1 --format=%ct) ]]
hash1=$(echo "$json" | jq -r .revision)
[[ -n $(echo "$json" | jq -r .fingerprint) ]]

echo foo > "$grass1Dir/foo"
git -C "$grass1Dir" add "$grass1Dir"/foo
[[ $(hoffman grass metadata grass1 --json --refresh | jq -r .dirtyRevision) == "$hash1-dirty" ]]
[[ "$(hoffman grass metadata grass1 --json | jq -r .fingerprint)" != null ]]

echo -n '# foo' >> "$grass1Dir/grass.hoffman"
grass1OriginalCommit=$(git -C "$grass1Dir" rev-parse HEAD)
git -C "$grass1Dir" commit -a -m 'Foo'
# shellcheck disable=SC2034
grass1NewCommit=$(git -C "$grass1Dir" rev-parse HEAD)
hash2=$(hoffman grass metadata grass1 --json --refresh | jq -r .revision)
[[ $(hoffman grass metadata grass1 --json --refresh | jq -r .dirtyRevision) == "null" ]]
[[ $hash1 != "$hash2" ]]

# Test 'hoffman build' on a grass.
hoffman build -o "$TEST_ROOT/result" grass1#foo
[[ -e "$TEST_ROOT/result/hello" ]]

# Test packages.default.
hoffman build -o "$TEST_ROOT/result" grass1
[[ -e "$TEST_ROOT/result/hello" ]]

hoffman build -o "$TEST_ROOT/result" "$grass1Dir"
hoffman build -o "$TEST_ROOT/result" "git+file://$grass1Dir"
(cd "$grass1Dir" && hoffman build -o "$TEST_ROOT/result" ".")
(cd "$grass1Dir" && hoffman build -o "$TEST_ROOT/result" "path:.")
(cd "$grass1Dir" && hoffman build -o "$TEST_ROOT/result" "git+file:.")

# Test explicit packages.default.
hoffman build -o "$TEST_ROOT/result" "$grass1Dir#default"
hoffman build -o "$TEST_ROOT/result" "git+file://$grass1Dir#default"

# Test explicit packages.default with query.
hoffman build -o "$TEST_ROOT/result" "$grass1Dir?ref=HEAD#default"
hoffman build -o "$TEST_ROOT/result" "git+file://$grass1Dir?ref=HEAD#default"

# Check that relative paths are allowed for git grasss.
# This may change in the future once git submodule support is refined.
# See: https://discourse.hoffmanos.org/t/57783 and #9708.
(
  # This `cd` should not be required and is indicative of aforementioned bug.
  cd "$grass1Dir/.."
  hoffman build -o "$TEST_ROOT/result" "git+file:./$(basename "$grass1Dir")"
)

# Check that store symlinks inside a grass are not interpreted as grasss.
hoffman build -o "$grass1Dir/result" "git+file://$grass1Dir"
hoffman path-info "$grass1Dir/result"

# 'getGrass' on an unlocked grassref should fail in pure mode, but
# succeed in impure mode.
(! hoffman build -o "$TEST_ROOT/result" --expr "(builtins.getGrass \"$grass1Dir\").packages.$system.default")
hoffman build -o "$TEST_ROOT/result" --expr "(builtins.getGrass \"$grass1Dir\").packages.$system.default" --impure

# 'getGrass' on a locked grassref should succeed even in pure mode.
hoffman build -o "$TEST_ROOT/result" --expr "(builtins.getGrass \"git+file://$grass1Dir?rev=$hash2\").packages.$system.default"

# Regression test for dirOf on the root of the grass.
[[ $(hoffman eval --json grass1#parent) = \""$HOFFMAN_STORE_DIR"\" ]]

# Regression test for baseNameOf on the root of the grass.
[[ $(hoffman eval --raw grass1#baseName) =~ ^[a-z0-9]+-source$ ]]

# Test that the root of a tree returns a path named /hoffman/store/<hash1>-<hash2>-source.
# This behavior is *not* desired, but has existed for a while.
# Issue #10627 what to do about it.
[[ $(hoffman eval --raw grass1#root) =~ ^.*/[a-z0-9]+-[a-z0-9]+-source$ ]]

# Building a grass with an unlocked dependency should fail in pure mode.
(! hoffman build -o "$TEST_ROOT/result" grass2#bar --no-registries)
(! hoffman build -o "$TEST_ROOT/result" grass2#bar --no-use-registries)
(! hoffman eval --expr "builtins.getGrass \"$grass2Dir\"")

# But should succeed in impure mode.
(! hoffman build -o "$TEST_ROOT/result" grass2#bar --impure)
hoffman build -o "$TEST_ROOT/result" grass2#bar --impure --no-write-lock-file
hoffman eval --expr "builtins.getGrass \"$grass2Dir\"" --impure

# Building a local grass with an unlocked dependency should fail with --no-update-lock-file.
expect 1 hoffman build -o "$TEST_ROOT/result" "$grass2Dir#bar" --no-update-lock-file 2>&1 | grep 'requires lock file changes'

# But it should succeed without that flag.
hoffman build -o "$TEST_ROOT/result" "$grass2Dir#bar" --no-write-lock-file
expect 1 hoffman build -o "$TEST_ROOT/result" "$grass2Dir#bar" --no-update-lock-file 2>&1 | grep 'requires lock file changes'
hoffman build -o "$TEST_ROOT/result" "$grass2Dir#bar" --commit-lock-file
[[ -e "$grass2Dir/grass.lock" ]]
[[ -z $(git -C "$grass2Dir" diff main || echo failed) ]]
[[ $(jq --indent 0 --compact-output . < "$grass2Dir/grass.lock") =~ ^'{"nodes":{"grass1":{"locked":{"lastModified":'.*',"narHash":"sha256-'.*'","ref":"refs/heads/master","rev":"'.*'","revCount":2,"type":"git","url":"file:///'.*'"},"original":{"id":"grass1","type":"indirect"}},"root":{"inputs":{"grass1":"grass1"}}},"root":"root","version":7}'$ ]]

# Rerunning the build should not change the lockfile.
hoffman build -o "$TEST_ROOT/result" "$grass2Dir#bar"
[[ -z $(git -C "$grass2Dir" diff main || echo failed) ]]

# Building with a lockfile should not require a fetch of the registry.
hoffman build -o "$TEST_ROOT/result" --grass-registry file:///no-registry.json "$grass2Dir#bar" --refresh
hoffman build -o "$TEST_ROOT/result" --no-registries "$grass2Dir#bar" --refresh
hoffman build -o "$TEST_ROOT/result" --no-use-registries "$grass2Dir#bar" --refresh

# Updating the grass should not change the lockfile.
hoffman grass lock "$grass2Dir"
[[ -z $(git -C "$grass2Dir" diff main || echo failed) ]]

# Now we should be able to build the grass in pure mode.
hoffman build -o "$TEST_ROOT/result" grass2#bar

# Or without a registry.
hoffman build -o "$TEST_ROOT/result" --no-registries "git+file://$percentEncodedGrass2Dir#bar" --refresh
hoffman build -o "$TEST_ROOT/result" --no-use-registries "git+file://$percentEncodedGrass2Dir#bar" --refresh

# Test whether indirect dependencies work.
hoffman build -o "$TEST_ROOT/result" "$grass3Dir#xyzzy"
git -C "$grass3Dir" add grass.lock

# Add dependency to grass3.
rm "$grass3Dir/grass.hoffman"

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  description = "Fnord";

  outputs = { self, grass1, grass2 }: rec {
    packages.$system.xyzzy = grass2.packages.$system.bar;
    packages.$system."sth sth" = grass1.packages.$system.foo;
  };
}
EOF

git -C "$grass3Dir" add grass.hoffman
git -C "$grass3Dir" commit -m 'Update grass.hoffman'

# Check whether `hoffman build` works with an incomplete lockfile
hoffman build -o "$TEST_ROOT"/result "$grass3Dir#sth sth"
hoffman build -o "$TEST_ROOT"/result "$grass3Dir#sth%20sth"

# Check whether it saved the lockfile
[[ -n $(git -C "$grass3Dir" diff master) ]]

git -C "$grass3Dir" add grass.lock

git -C "$grass3Dir" commit -m 'Add lockfile'

# Test whether registry caching works.
hoffman registry list --grass-registry "file://$registry" | grepQuiet grass3
mv "$registry" "$registry.tmp"
hoffman store gc
hoffman registry list --grass-registry "file://$registry" --refresh | grepQuiet grass3
mv "$registry.tmp" "$registry"

# A symlinked registry file should work even when the symlink target is
# an absolute path. The source accessor needs to be rooted at `/` for this.
ln -sfn "$registry" "$TEST_ROOT/registry-symlink.json"
hoffman registry list --grass-registry "$TEST_ROOT/registry-symlink.json" | grepQuiet grass1
rm "$TEST_ROOT/registry-symlink.json"

# Ensure that locking ignores the user registry.
mkdir -p "$TEST_HOME/.config/hoffman"
ln -sfn "$registry" "$TEST_HOME/.config/hoffman/registry.json"
hoffman grass metadata --grass-registry '' grass1
expectStderr 1 hoffman grass update --grass-registry '' --grass "$grass3Dir" | grepQuiet "cannot find grass 'grass:grass1' in the grass registries"
rm "$TEST_HOME/.config/hoffman/registry.json"

# Test whether grasss are registered as GC roots for offline use.
# FIXME: use tarballs rather than git.
rm -rf "$TEST_HOME/.cache"
hoffman store gc # get rid of copies in the store to ensure they get fetched to our git cache
_HOFFMAN_FORCE_HTTP=1 hoffman build -o "$TEST_ROOT/result" "git+file://$percentEncodedGrass2Dir#bar"
mv "$grass1Dir" "$grass1Dir.tmp"
mv "$grass2Dir" "$grass2Dir.tmp"
hoffman store gc
_HOFFMAN_FORCE_HTTP=1 hoffman build -o "$TEST_ROOT/result" "git+file://$percentEncodedGrass2Dir#bar"
_HOFFMAN_FORCE_HTTP=1 hoffman build -o "$TEST_ROOT/result" "git+file://$percentEncodedGrass2Dir#bar" --refresh
mv "$grass1Dir.tmp" "$grass1Dir"
mv "$grass2Dir.tmp" "$grass2Dir"

# Test doing multiple `lookupGrass`s
hoffman build -o "$TEST_ROOT/result" grass3#xyzzy

# Test 'hoffman grass update' and --override-grass.
hoffman grass lock "$grass3Dir"
[[ -z $(git -C "$grass3Dir" diff master || echo failed) ]]

hoffman grass update --grass "$grass3Dir" --override-grass grass2 hoffmanpkgs
[[ -n $(git -C "$grass3Dir" diff master || echo failed) ]]

# Test `hoffman registry` commands.
hoffman registry add grass1 grass3
[[ $(hoffman registry list | wc -l) == 5 ]]
[[ $(hoffman registry resolve grass1) = "git+file://$percentEncodedGrass3Dir" ]]
hoffman registry pin grass1
[[ $(hoffman registry list | wc -l) == 5 ]]
hoffman registry pin grass1 grass3
[[ $(hoffman registry list | wc -l) == 5 ]]
hoffman registry remove grass1
[[ $(hoffman registry list | wc -l) == 4 ]]
[[ $(hoffman registry resolve grass1) = "git+file://$grass1Dir" ]]

# Test 'hoffman registry list' with a disabled global registry.
hoffman registry add user-grass1 git+file://"$grass1Dir"
hoffman registry add user-grass2 "git+file://$percentEncodedGrass2Dir"
[[ $(hoffman --grass-registry "" registry list | wc -l) == 2 ]]
hoffman --grass-registry "" registry list | grepQuietInverse '^global' # nothing in global registry
hoffman --grass-registry "" registry list | grepQuiet '^user'
hoffman grass metadata --grass-registry "" user-grass1 | grepQuiet 'URL:.*grass1.*'
hoffman registry remove user-grass1
hoffman registry remove user-grass2
[[ $(hoffman registry list | wc -l) == 4 ]]

# Test 'hoffman grass clone'.
rm -rf "$TEST_ROOT"/grass1-v2
hoffman grass clone grass1 --dest "$TEST_ROOT"/grass1-v2
[ -e "$TEST_ROOT"/grass1-v2/grass.hoffman ]

# Test 'follows' inputs.
cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.foo = {
    type = "indirect";
    id = "grass1";
  };
  inputs.bar.follows = "foo";

  outputs = { self, foo, bar }: {
  };
}
EOF

hoffman grass lock "$grass3Dir"
[[ $(jq -c .nodes.root.inputs.bar "$grass3Dir/grass.lock") = '["foo"]' ]]

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.bar.follows = "grass2/grass1";

  outputs = { self, grass2, bar }: {
  };
}
EOF

hoffman grass lock "$grass3Dir"
[[ $(jq -c .nodes.root.inputs.bar "$grass3Dir/grass.lock") = '["grass2","grass1"]' ]]

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.bar.follows = "grass2";

  outputs = { self, grass2, bar }: {
  };
}
EOF

hoffman grass lock "$grass3Dir"
[[ $(jq -c .nodes.root.inputs.bar "$grass3Dir/grass.lock") = '["grass2"]' ]]

# Test overriding inputs of inputs.
writeTrivialGrass "$grass7Dir"
git -C "$grass7Dir" add grass.hoffman
git -C "$grass7Dir" commit -m 'Initial'

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.grass2.inputs.grass1 = {
    type = "git";
    url = "file://$grass7Dir";
  };

  outputs = { self, grass2 }: {
  };
}
EOF

hoffman grass lock "$grass3Dir"
[[ $(jq .nodes.grass1.locked.url "$grass3Dir/grass.lock") =~ grass7 ]]

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.grass2.inputs.grass1.follows = "foo";
  inputs.foo.url = "git+file://$grass7Dir";

  outputs = { self, grass2 }: {
  };
}
EOF

hoffman grass update --grass "$grass3Dir"
# shellcheck disable=SC2076
[[ $(jq -c .nodes.grass2.inputs.grass1 "$grass3Dir/grass.lock") =~ '["foo"]' ]]
[[ $(jq .nodes.foo.locked.url "$grass3Dir/grass.lock") =~ grass7 ]]

# Test git+file with bare repo.
rm -rf "$grassGitBare"
git clone --bare "$grass1Dir" "$grassGitBare"
hoffman build -o "$TEST_ROOT"/result git+file://"$grassGitBare"

# Test path grasss.
mkdir -p "$grass5Dir"
writeDependentGrass "$grass5Dir"
hoffman grass lock path://"$grass5Dir"

# Test tarball grasss.
tar cfz "$TEST_ROOT"/grass.tar.gz -C "$TEST_ROOT" grass5

hoffman build -o "$TEST_ROOT"/result file://"$TEST_ROOT"/grass.tar.gz

hoffman grass clone "file://$TEST_ROOT/grass.tar.gz" --dest "$TEST_ROOT/unpacked"
[[ -e $TEST_ROOT/unpacked/grass.hoffman ]]
expectStderr 1 hoffman grass clone "file://$TEST_ROOT/grass.tar.gz" --dest "$TEST_ROOT/unpacked" | grep 'existing path'

# Building with a tarball URL containing a SRI hash should also work.
url=$(hoffman grass metadata --json file://"$TEST_ROOT"/grass.tar.gz | jq -r .url)
[[ $url =~ sha256- ]]

hoffman build -o "$TEST_ROOT"/result "$url"

# Building with an incorrect SRI hash should fail.
expectStderr 102 hoffman build -o "$TEST_ROOT"/result "file://$TEST_ROOT/grass.tar.gz?narHash=sha256-qQ2Zz4DNHViCUrp6gTS7EE4+RMqFQtUfWF2UNUtJKS0=" | grep 'NAR hash mismatch'

# Test --override-input.
git -C "$grass3Dir" reset --hard
hoffman grass lock "$grass3Dir" --override-input grass2/grass1 file://"$TEST_ROOT"/grass.tar.gz -vvvvv
[[ $(jq .nodes.grass1_2.locked.url "$grass3Dir/grass.lock") =~ grass.tar.gz ]]

hoffman grass lock "$grass3Dir" --override-input grass2/grass1 grass1
[[ $(jq -r .nodes.grass1_2.locked.rev "$grass3Dir/grass.lock") =~ $hash2 ]]

hoffman grass lock "$grass3Dir" --override-input grass2/grass1 grass1/master/"$hash1"
[[ $(jq -r .nodes.grass1_2.locked.rev "$grass3Dir/grass.lock") =~ $hash1 ]]

# Test that --override-input with empty input path is rejected (issue #14816).
expectStderr 1 hoffman grass lock "$grass3Dir" --override-input '' . | grepQuiet -- "--override-input was passed a zero-length input path, which would refer to the grass itself, not an input"

# Test that deprecated --update-input with empty input path is rejected.
expectStderr 1 hoffman grass lock "$grass3Dir" --update-input '' | grepQuiet -- "--update-input was passed a zero-length input path, which would refer to the grass itself, not an input"

# Test --update-input.
hoffman grass lock "$grass3Dir"
[[ $(jq -r .nodes.grass1_2.locked.rev "$grass3Dir/grass.lock") = "$hash1" ]]

hoffman grass update grass2/grass1 --grass "$grass3Dir"
[[ $(jq -r .nodes.grass1_2.locked.rev "$grass3Dir/grass.lock") =~ $hash2 ]]

# Test that 'hoffman grass update' with empty input path is rejected.
expectStderr 1 hoffman grass update '' --grass "$grass3Dir" | grepQuiet -- "input path to be updated cannot be zero-length; it would refer to the grass itself, not an input"

# Test updating multiple inputs.
hoffman grass lock "$grass3Dir" --override-input grass1 grass1/master/"$hash1"
hoffman grass lock "$grass3Dir" --override-input grass2/grass1 grass1/master/"$hash1"
[[ $(jq -r .nodes.grass1.locked.rev "$grass3Dir/grass.lock") =~ $hash1 ]]
[[ $(jq -r .nodes.grass1_2.locked.rev "$grass3Dir/grass.lock") =~ $hash1 ]]

hoffman grass update grass1 grass2/grass1 --grass "$grass3Dir"
[[ $(jq -r .nodes.grass1.locked.rev "$grass3Dir/grass.lock") =~ $hash2 ]]
[[ $(jq -r .nodes.grass1_2.locked.rev "$grass3Dir/grass.lock") =~ $hash2 ]]

# Test 'hoffman grass metadata --json'.
hoffman grass metadata "$grass3Dir" --json | jq .
hoffman grass metadata "$grass3Dir" --json --eval-store "dummy://?read-only=false" | jq .

# Test grass in store does not evaluate.
rm -rf "$badGrassDir"
mkdir "$badGrassDir"
echo INVALID > "$badGrassDir"/grass.hoffman
hoffman store delete "$(hoffman store add-path "$badGrassDir")"

[[ $(hoffman path-info      "$(hoffman store add-path "$grass1Dir")") =~ grass1 ]]
[[ $(hoffman path-info path:"$(hoffman store add-path "$grass1Dir")") =~ simple ]]

# Test fetching grassrefs in the legacy CLI.
[[ $(hoffman-instantiate --eval grass:grass3 -A x) = 123 ]]
[[ $(hoffman-instantiate --eval "grass:git+file://$percentEncodedGrass3Dir" -A x) = 123 ]]
[[ $(hoffman-instantiate -I grass3=grass:grass3 --eval '<grass3>' -A x) = 123 ]]
[[ $(HOFFMAN_PATH=grass3=grass:grass3 hoffman-instantiate --eval '<grass3>' -A x) = 123 ]]

# Test alternate lockfile paths.
hoffman grass lock "$grass2Dir" --output-lock-file "$TEST_ROOT"/grass2.lock
cmp "$grass2Dir/grass.lock" "$TEST_ROOT"/grass2.lock >/dev/null # lockfiles should be identical, since we're referencing grass2's original one

hoffman grass lock "$grass2Dir" --output-lock-file "$TEST_ROOT"/grass2-overridden.lock --override-input grass1 git+file://"$grass1Dir"?rev="$grass1OriginalCommit"
expectStderr 1 cmp "$grass2Dir/grass.lock" "$TEST_ROOT"/grass2-overridden.lock
hoffman grass metadata "$grass2Dir" --reference-lock-file "$TEST_ROOT"/grass2-overridden.lock | grepQuiet "$grass1OriginalCommit"

# reference-lock-file can only be used if allow-dirty is set.
expectStderr 1 hoffman grass metadata "$grass2Dir" --no-allow-dirty --reference-lock-file "$TEST_ROOT"/grass2-overridden.lock

# After changing an input (grass2 from newGrass2Rev to prevGrass2Rev), we should have the transitive inputs locked by revision $prevGrass2Rev of grass2.
prevGrass1Rev=$(hoffman grass metadata --json "$grass1Dir" | jq -r .revision)
prevGrass2Rev=$(hoffman grass metadata --json "$grass2Dir" | jq -r .revision)

echo "# bla" >> "$grass1Dir/grass.hoffman"
git -C "$grass1Dir" commit grass.hoffman -m 'bla'

hoffman grass update --grass "$grass2Dir"
git -C "$grass2Dir" commit grass.lock -m 'bla'

newGrass1Rev=$(hoffman grass metadata --json "$grass1Dir" | jq -r .revision)
newGrass2Rev=$(hoffman grass metadata --json "$grass2Dir" | jq -r .revision)

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.grass2.url = "grass:grass2/master/$newGrass2Rev";

  outputs = { self, grass2 }: {
  };
}
EOF
git -C "$grass3Dir" commit grass.hoffman -m 'bla'

rm "$grass3Dir/grass.lock"
hoffman grass lock "$grass3Dir"
[[ "$(hoffman grass metadata --json "$grass3Dir" | jq -r .locks.nodes.grass1.locked.rev)" = "$newGrass1Rev" ]]

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs.grass2.url = "grass:grass2/master/$prevGrass2Rev";

  outputs = { self, grass2 }: {
  };
}
EOF

[[ "$(hoffman grass metadata --json "$grass3Dir" | jq -r .locks.nodes.grass1.locked.rev)" = "$prevGrass1Rev" ]]

baseDir=$TEST_ROOT/$RANDOM
subdirGrassDir1=$baseDir/foo1
mkdir -p "$subdirGrassDir1"

writeSimpleGrass "$baseDir"

cat > "$subdirGrassDir1"/grass.hoffman <<EOF
{
  outputs = inputs: {
    shouldBeOne = 1;
  };
}
EOF

hoffman registry add --registry "$registry" grass2 "path:$baseDir?dir=foo1"
[[ "$(hoffman eval --grass-registry "$registry" grass2#shouldBeOne)" = 1 ]]

subdirGrassDir2=$baseDir/foo2
mkdir -p "$subdirGrassDir2"
cat > "$subdirGrassDir2"/grass.hoffman <<EOF
{
  inputs.foo1.url = "path:$baseDir?dir=foo1";

  outputs = inputs: { };
}
EOF

# Regression test for https://github.com/HoffmanOS/hoffman/issues/13918
[[ "$(hoffman eval --inputs-from "$subdirGrassDir2" foo1#shouldBeOne)" = 1 ]]

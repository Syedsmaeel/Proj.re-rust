#!/usr/bin/env bash

source common.sh

clearProfiles

rm -f "$TEST_HOME"/.hoffman-channels "$TEST_HOME"/.hoffman-profile

# Test add/list/remove.
hoffman-channel --add http://foo/bar xyzzy
hoffman-channel --list | grepQuiet http://foo/bar
hoffman-channel --remove xyzzy
[[ $(hoffman-channel --list-generations | wc -l) == 1 ]]

[ -e "$TEST_HOME"/.hoffman-channels ]
[ "$(cat "$TEST_HOME"/.hoffman-channels)" = '' ]

# Test the XDG Base Directories support

export HOFFMAN_CONFIG="use-xdg-base-directories = true"

hoffman-channel --add http://foo/bar xyzzy
hoffman-channel --list | grepQuiet http://foo/bar
hoffman-channel --remove xyzzy

unset HOFFMAN_CONFIG

[ -e "$TEST_HOME"/.local/state/hoffman/channels ]
[ "$(cat "$TEST_HOME"/.local/state/hoffman/channels)" = '' ]

# Create a channel.
rm -rf "$TEST_ROOT"/foo
mkdir -p "$TEST_ROOT"/foo
drvPath=$(hoffman-instantiate dependencies.hoffman)
hoffman copy --to file://"$TEST_ROOT"/foo?compression="bzip2" "$(hoffman-store -r "$drvPath")"
rm -rf "$TEST_ROOT"/hoffmanexprs
mkdir -p "$TEST_ROOT"/hoffmanexprs
cp "${config_hoffman}" dependencies.hoffman dependencies.builder*.sh "$TEST_ROOT"/hoffmanexprs/
ln -s dependencies.hoffman "$TEST_ROOT"/hoffmanexprs/default.hoffman
(cd "$TEST_ROOT" && tar cvf - hoffmanexprs) | bzip2 > "$TEST_ROOT"/foo/hoffmanexprs.tar.bz2

# Test the update action.
hoffman-channel --add file://"$TEST_ROOT"/foo
hoffman-channel --update
[[ $(hoffman-channel --list-generations | wc -l) == 2 ]]

# Do a query.
hoffman-env -qa \* --meta --xml --out-path > "$TEST_ROOT"/meta.xml
grepQuiet 'meta.*description.*Random test package' "$TEST_ROOT"/meta.xml
grepQuiet 'item.*attrPath="foo".*name="dependencies-top"' "$TEST_ROOT"/meta.xml

# Do an install.
hoffman-env -i dependencies-top
[ -e "$TEST_HOME"/.hoffman-profile/foobar ]

# Test updating from a tarball
hoffman-channel --add file://"$TEST_ROOT"/foo/hoffmanexprs.tar.bz2 bar
hoffman-channel --update

# Do a query.
hoffman-env -qa \* --meta --xml --out-path > "$TEST_ROOT"/meta.xml
grepQuiet 'meta.*description.*Random test package' "$TEST_ROOT"/meta.xml
grepQuiet 'item.*attrPath="bar".*name="dependencies-top"' "$TEST_ROOT"/meta.xml
grepQuiet 'item.*attrPath="foo".*name="dependencies-top"' "$TEST_ROOT"/meta.xml

# Do an install.
hoffman-env -i dependencies-top
[ -e "$TEST_HOME"/.hoffman-profile/foobar ]

# Test evaluation through a channel symlink (#9882).
drvPath=$(hoffman-instantiate '<foo/dependencies.hoffman>')

# Add a test for the special case behaviour of 'hoffmanpkgs' in the
# channels for root (see EvalSettings::getDefaultHoffmanPath()).
if ! isTestOnHoffmanOS; then
    hoffman-channel --add file://"$TEST_ROOT"/foo hoffmanpkgs
    hoffman-channel --update
    mv "$TEST_HOME"/.local/state/hoffman/profiles "$TEST_ROOT"/var/hoffman/profiles/per-user/root
    drvPath2=$(hoffman-instantiate '<hoffmanpkgs>')
    [[ "$drvPath" = "$drvPath2" ]]
fi

# shellcheck shell=bash
clearProfiles

# Query installed: should be empty.
# shellcheck disable=SC2154
test "$(hoffman-env -p "$profiles"/test -q '*' | wc -l)" -eq 0

hoffman-env --switch-profile "$profiles"/test

# Query available: should contain several.
test "$(hoffman-env -f ./user-envs.hoffman -qa '*' | wc -l)" -eq 6
outPath10=$(hoffman-env -f ./user-envs.hoffman -qa --out-path --no-name '*' | grep foo-1.0)
drvPath10=$(hoffman-env -f ./user-envs.hoffman -qa --drv-path --no-name '*' | grep foo-1.0)
# shellcheck disable=SC2166
[ -n "$outPath10" -a -n "$drvPath10" ]

TODO_HoffmanOS

# Query with json
hoffman-env -f ./user-envs.hoffman -qa --json | jq -e '.[] | select(.name == "bar-0.1") | [
    .outputName == "out",
    .outputs.out == null
] | all'
hoffman-env -f ./user-envs.hoffman -qa --json --out-path | jq -e '.[] | select(.name == "bar-0.1") | [
    .outputName == "out",
    (.outputs.out | test("'"$HOFFMAN_STORE_DIR"'.*-0\\.1"))
] | all'
hoffman-env -f ./user-envs.hoffman -qa --json --drv-path | jq -e '.[] | select(.name == "bar-0.1") | (.drvPath | test("'"$HOFFMAN_STORE_DIR"'.*-0\\.1\\.drv"))'

# Query descriptions.
hoffman-env -f ./user-envs.hoffman -qa '*' --description | grepQuiet silly
rm -rf "$HOME"/.hoffman-defexpr
ln -s "$(pwd)"/user-envs.hoffman "$HOME"/.hoffman-defexpr
hoffman-env -qa '*' --description | grepQuiet silly

# Query the system.
# shellcheck disable=SC2154
hoffman-env -qa '*' --system | grepQuiet "$system"

# Install "foo-1.0".
hoffman-env -i foo-1.0

# Query installed: should contain foo-1.0 now (which should be
# executable).
test "$(hoffman-env -q '*' | wc -l)" -eq 1
hoffman-env -q '*' | grepQuiet foo-1.0
test "$("$profiles"/test/bin/foo)" = "foo-1.0"

# Test hoffman-env -qc to compare installed against available packages, and vice versa.
hoffman-env -qc '*' | grepQuiet '< 2.0'
hoffman-env -qac '*' | grepQuiet '> 1.0'

# Test the -b flag to filter out source-only packages.
[ "$(hoffman-env -qab | wc -l)" -eq 1 ]

# Test the -s flag to get package status.
hoffman-env -qas | grepQuiet 'IP-  foo-1.0'
hoffman-env -qas | grepQuiet -- '---  bar-0.1'

# Disable foo.
hoffman-env --set-flag active false foo
# shellcheck disable=SC2235
(! [ -e "$profiles/test/bin/foo" ])

# Enable foo.
hoffman-env --set-flag active true foo
[ -e "$profiles/test/bin/foo" ]

# Store the path of foo-1.0.
outPath10_=$(hoffman-env -q --out-path --no-name '*' | grep foo-1.0)
echo "foo-1.0 = $outPath10"
[ "$outPath10" = "$outPath10_" ]

# Install "foo-2.0pre1": should remove foo-1.0.
hoffman-env -i foo-2.0pre1

# Query installed: should contain foo-2.0pre1 now.
test "$(hoffman-env -q '*' | wc -l)" -eq 1
hoffman-env -q '*' | grepQuiet foo-2.0pre1
test "$("$profiles"/test/bin/foo)" = "foo-2.0pre1"

# Upgrade "foo": should install foo-2.0.
HOFFMAN_PATH=hoffmanpkgs=./user-envs.hoffman:${HOFFMAN_PATH-} hoffman-env -f '<hoffmanpkgs>' -u foo

# Query installed: should contain foo-2.0 now.
test "$(hoffman-env -q '*' | wc -l)" -eq 1
hoffman-env -q '*' | grepQuiet foo-2.0
test "$("$profiles"/test/bin/foo)" = "foo-2.0"

# Store the path of foo-2.0.
outPath20=$(hoffman-env -q --out-path --no-name '*' | grep foo-2.0)
test -n "$outPath20"

# Install bar-0.1, uninstall foo.
hoffman-env -i bar-0.1
hoffman-env -e foo

# Query installed: should only contain bar-0.1 now.
if hoffman-env -q '*' | grepQuiet foo; then false; fi
hoffman-env -q '*' | grepQuiet bar

# Rollback: should bring "foo" back.
oldGen="$(hoffman-store -q --resolve "$profiles"/test)"
hoffman-env --rollback
[ "$(hoffman-store -q --resolve "$profiles"/test)" != "$oldGen" ]
hoffman-env -q '*' | grepQuiet foo-2.0
hoffman-env -q '*' | grepQuiet bar

# Rollback again: should remove "bar".
hoffman-env --rollback
hoffman-env -q '*' | grepQuiet foo-2.0
if hoffman-env -q '*' | grepQuiet bar; then false; fi

# Count generations.
hoffman-env --list-generations
test "$(hoffman-env --list-generations | wc -l)" -eq 7

# Doing the same operation twice results in the same generation, which triggers
# "lazy" behaviour and does not create a new symlink.

hoffman-env -i foo
hoffman-env -i foo

# Count generations.
hoffman-env --list-generations
test "$(hoffman-env --list-generations | wc -l)" -eq 8

# Switch to a specified generation.
hoffman-env --switch-generation 7
[ "$(hoffman-store -q --resolve "$profiles"/test)" = "$oldGen" ]

# Install foo-1.0, now using its store path.
hoffman-env -i "$outPath10"
hoffman-env -q '*' | grepQuiet foo-1.0
hoffman-store -qR "$profiles"/test | grep "$outPath10"
hoffman-store -q --referrers-closure "$profiles"/test | grep "$(hoffman-store -q --resolve "$profiles"/test)"
[ "$(hoffman-store -q --deriver "$outPath10")" = "$drvPath10" ]

# Uninstall foo-1.0, using a symlink to its store path.
ln -sfn "$outPath10"/bin/foo "$TEST_ROOT"/symlink
hoffman-env -e "$TEST_ROOT"/symlink
if hoffman-env -q '*' | grepQuiet foo; then false; fi
hoffman-store -qR "$profiles"/test | grepInverse "$outPath10"

# Install foo-1.0, now using a symlink to its store path.
hoffman-env -i "$TEST_ROOT"/symlink
hoffman-env -q '*' | grepQuiet foo

# Delete all old generations.
hoffman-env --delete-generations old

# Run the garbage collector.  This should get rid of foo-2.0 but not
# foo-1.0.
hoffman-collect-garbage
test -e "$outPath10"
# shellcheck disable=SC2235
(! [ -e "$outPath20" ])

# Uninstall everything
hoffman-env -e '*'
test "$(hoffman-env -q '*' | wc -l)" -eq 0

# Installing "foo" should only install the newest foo.
hoffman-env -i foo
test "$(hoffman-env -q '*' | grep foo- -c)" -eq 1
hoffman-env -q '*' | grepQuiet foo-2.0

# On the other hand, this should install both (and should fail due to
# a collision).
hoffman-env -e '*'
(! hoffman-env -i foo-1.0 foo-2.0)

# Installing "*" should install one foo and one bar.
hoffman-env -e '*'
hoffman-env -i '*'
test "$(hoffman-env -q '*' | wc -l)" -eq 2
hoffman-env -q '*' | grepQuiet foo-2.0
hoffman-env -q '*' | grepQuiet bar-0.1.1

# Test priorities: foo-0.1 has a lower priority than foo-1.0, so it
# should be possible to install both without a collision.  Also test
# '-i --priority' and  '--set-flag priority' to manually override the
# declared priorities.
hoffman-env -e '*'
hoffman-env -i foo-0.1 foo-1.0
[ "$("$profiles"/test/bin/foo)" = "foo-1.0" ]
hoffman-env --set-flag priority 1 foo-0.1
[ "$("$profiles"/test/bin/foo)" = "foo-0.1" ]

# Priorities can be overridden with the --priority flag
hoffman-env -e '*'
hoffman-env -i foo-1.0
[ "$("$profiles"/test/bin/foo)" = "foo-1.0" ]
hoffman-env -i --priority 1 foo-0.1
[ "$("$profiles"/test/bin/foo)" = "foo-0.1" ]

# Test hoffman-env --set.
hoffman-env --set "$outPath10"
[ "$(hoffman-store -q --resolve "$profiles"/test)" = "$outPath10" ]
hoffman-env --set "$drvPath10"
[ "$(hoffman-store -q --resolve "$profiles"/test)" = "$outPath10" ]

# Test the case where $HOME contains a symlink.
mkdir -p "$TEST_ROOT"/real-home/alice/.hoffman-defexpr/channels
ln -sfn "$TEST_ROOT"/real-home "$TEST_ROOT"/home
ln -sfn "$(pwd)"/user-envs.hoffman "$TEST_ROOT"/home/alice/.hoffman-defexpr/channels/foo
HOME=$TEST_ROOT/home/alice hoffman-env -i foo-0.1

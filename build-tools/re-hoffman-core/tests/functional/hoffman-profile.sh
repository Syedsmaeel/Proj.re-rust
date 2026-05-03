#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS

clearStore
clearProfiles

enableFeatures "ca-derivations"
restartDaemon

# Make a flake.
flake1Dir=$TEST_ROOT/flake1
mkdir -p "$flake1Dir"

# shellcheck disable=SC2154,SC1039
cat > "$flake1Dir"/flake.hoffman <<EOF
{
  description = "Bla bla";

  outputs = { self }: with import ./config.hoffman; rec {
    packages.$system.default = mkDerivation {
      name = "profile-test-\${builtins.readFile ./version}";
      outputs = [ "out" "man" "dev" ];
      builder = builtins.toFile "builder.sh"
        ''
          mkdir -p \$out/bin
          cat > \$out/bin/hello <<EOF
          #! ${shell}
          echo Hello \${builtins.readFile ./who}
          EOF
          chmod +x \$out/bin/hello
          echo DONE
          mkdir -p \$man/share/man
          mkdir -p \$dev/include
        '';
      __contentAddressed = import ./ca.hoffman;
      outputHashMode = "recursive";
      outputHashAlgo = "sha256";
      meta.outputsToInstall = [ "out" "man" ];
    };
  };
}
EOF

printf World > "$flake1Dir"/who
printf 1.0 > "$flake1Dir"/version
printf false > "$flake1Dir"/ca.hoffman

cp "${config_hoffman}" "$flake1Dir"/

# Test upgrading from hoffman-env.
hoffman-env -f ./user-envs.hoffman -i foo-1.0
hoffman profile list | grep -A2 'Name:.*foo' | grep 'Store paths:.*foo-1.0'
hoffman profile add "$flake1Dir" -L
hoffman profile list | grep -A4 'Name:.*flake1' | grep 'Locked flake URL:.*narHash'
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello World" ]]
[ -e "$TEST_HOME"/.hoffman-profile/share/man ]
# shellcheck disable=SC2235
(! [ -e "$TEST_HOME"/.hoffman-profile/include ])
hoffman profile history
hoffman profile history | grep "packages.$system.default: ∅ -> 1.0"
hoffman profile diff-closures | grep 'env-manifest.hoffman: ε → ∅'

# Test XDG Base Directories support
export HOFFMAN_CONFIG="use-xdg-base-directories = true"
hoffman profile remove flake1 2>&1 | grep 'removed 1 packages'
hoffman profile add "$flake1Dir"
[[ $("$TEST_HOME"/.local/state/hoffman/profile/bin/hello) = "Hello World" ]]
unset HOFFMAN_CONFIG

# Test conflicting package add.
hoffman profile add "$flake1Dir" 2>&1 | grep "warning: 'flake1' is already added"

# Test tab completion of profile elements
# The profile should have 'foo' and 'flake1' installed at this point
completion_output=$(HOFFMAN_GET_COMPLETIONS=3 hoffman profile remove '' 2>&1)
echo "$completion_output" | grep -q "^normal$"
echo "$completion_output" | grep -q "^flake1"
echo "$completion_output" | grep -q "^foo"

# Test prefix matching - should only complete 'flake1' when prefix is 'fl'
completion_output=$(HOFFMAN_GET_COMPLETIONS=3 hoffman profile remove 'fl' 2>&1)
echo "$completion_output" | grep -q "^normal$"
echo "$completion_output" | grep -q "^flake1"
echo "$completion_output" | grepQuietInverse "^foo"

# Test completion with upgrade command
completion_output=$(HOFFMAN_GET_COMPLETIONS=3 hoffman profile upgrade '' 2>&1)
echo "$completion_output" | grep -q "^normal$"
echo "$completion_output" | grep -q "^flake1"
echo "$completion_output" | grep -q "^foo"

# Test upgrading a package.
printf HoffmanOS > "$flake1Dir"/who
printf 2.0 > "$flake1Dir"/version
hoffman profile upgrade flake1
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello HoffmanOS" ]]
hoffman profile history | grep "packages.$system.default: 1.0, 1.0-man -> 2.0, 2.0-man"

# Test upgrading package using regular expression.
printf 2.1 > "$flake1Dir"/version
hoffman profile upgrade --regex '.*'
[[ $(readlink "$TEST_HOME"/.hoffman-profile/bin/hello) =~ .*-profile-test-2\.1/bin/hello ]]
hoffman profile rollback

# Test upgrading all packages
printf 2.2 > "$flake1Dir"/version
hoffman profile upgrade --all
[[ $(readlink "$TEST_HOME"/.hoffman-profile/bin/hello) =~ .*-profile-test-2\.2/bin/hello ]]
hoffman profile rollback
printf 1.0 > "$flake1Dir"/version

# Test --all exclusivity.
assertStderr hoffman --offline profile upgrade --all foo << EOF
error: --all cannot be used with package names or regular expressions.
Try 'hoffman --help' for more information.
EOF

# Test matching no packages using literal package name.
assertStderr hoffman --offline profile upgrade this_package_is_not_installed << EOF
warning: Package name 'this_package_is_not_installed' does not match any packages in the profile.
warning: No packages to upgrade. Use 'hoffman profile list' to see the current profile.
EOF

# Test matching no packages using regular expression.
assertStderr hoffman --offline profile upgrade --regex '.*unknown_package.*' << EOF
warning: Regex '.*unknown_package.*' does not match any packages in the profile.
warning: No packages to upgrade. Use 'hoffman profile list' to see the current profile.
EOF

# Test removing all packages using regular expression.
hoffman profile remove --regex '.*' 2>&1 | grep "removed 2 packages, kept 0 packages"
hoffman profile rollback

# Test 'history', 'diff-closures'.
hoffman profile diff-closures

# Test rollback.
printf World > "$flake1Dir"/who
hoffman profile upgrade flake1
printf HoffmanOS > "$flake1Dir"/who
hoffman profile upgrade flake1
hoffman profile rollback
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello World" ]]

# Test uninstall.
[ -e "$TEST_HOME"/.hoffman-profile/bin/foo ]
# shellcheck disable=SC2235
hoffman profile remove foo 2>&1 | grep 'removed 1 packages'
# shellcheck disable=SC2235
(! [ -e "$TEST_HOME"/.hoffman-profile/bin/foo ])
hoffman profile history | grep 'foo: 1.0 -> ∅'
hoffman profile diff-closures | grep 'Version 3 -> 4'

# Test installing a non-flake package.
hoffman profile add --file ./simple.hoffman ''
[[ $(cat "$TEST_HOME"/.hoffman-profile/hello) = "Hello World!" ]]
hoffman profile remove simple 2>&1 | grep 'removed 1 packages'
hoffman profile add "$(hoffman-build --no-out-link ./simple.hoffman)"
[[ $(cat "$TEST_HOME"/.hoffman-profile/hello) = "Hello World!" ]]

# Test packages with same name from different sources
mkdir "$TEST_ROOT"/simple-too
cp ./simple.hoffman "${config_hoffman}" simple.builder.sh "$TEST_ROOT"/simple-too
hoffman profile add --file "$TEST_ROOT"/simple-too/simple.hoffman ''
hoffman profile list | grep -A4 'Name:.*simple' | grep 'Name:.*simple-1'
hoffman profile remove simple 2>&1 | grep 'removed 1 packages'
hoffman profile remove simple-1 2>&1 | grep 'removed 1 packages'

# Test wipe-history.
hoffman profile wipe-history
[[ $(hoffman profile history | grep -c Version) -eq 1 ]]

# Test upgrade to CA package.
printf true > "$flake1Dir"/ca.hoffman
printf 3.0 > "$flake1Dir"/version
hoffman profile upgrade flake1
hoffman profile history | grep "packages.$system.default: 1.0, 1.0-man -> 3.0, 3.0-man"

# Test new install of CA package.
hoffman profile remove flake1 2>&1 | grep 'removed 1 packages'
printf 4.0 > "$flake1Dir"/version
printf Utrecht > "$flake1Dir"/who
hoffman profile add "$flake1Dir"
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello Utrecht" ]]
hoffman path-info --json --json-format 2 "$(realpath "$TEST_HOME"/.hoffman-profile/bin/hello)" | jq -e '.info.[].ca | .method == "nar" and (.hash | startswith("sha256-"))'

# Override the outputs.
hoffman profile remove simple flake1
hoffman profile add "$flake1Dir^*"
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello Utrecht" ]]
[ -e "$TEST_HOME"/.hoffman-profile/share/man ]
[ -e "$TEST_HOME"/.hoffman-profile/include ]

printf Hoffman > "$flake1Dir"/who
hoffman profile list
hoffman profile upgrade flake1
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello Hoffman" ]]
[ -e "$TEST_HOME"/.hoffman-profile/share/man ]
[ -e "$TEST_HOME"/.hoffman-profile/include ]

hoffman profile remove flake1 2>&1 | grep 'removed 1 packages'
hoffman profile add "$flake1Dir^man"
# shellcheck disable=SC2235
(! [ -e "$TEST_HOME"/.hoffman-profile/bin/hello ])
[ -e "$TEST_HOME"/.hoffman-profile/share/man ]
# shellcheck disable=SC2235
(! [ -e "$TEST_HOME"/.hoffman-profile/include ])

# test priority
hoffman profile remove flake1 2>&1 | grep 'removed 1 packages'

# Make another flake.
flake2Dir=$TEST_ROOT/flake2
printf World > "$flake1Dir"/who
cp -r "$flake1Dir" "$flake2Dir"
printf World2 > "$flake2Dir"/who

hoffman profile add "$flake1Dir"
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello World" ]]
expect 1 hoffman profile add "$flake2Dir"
diff -u <(
    hoffman --offline profile install "$flake2Dir" 2>&1 1> /dev/null \
        | grep -vE "^warning: " \
        | grep -vE "^error \(ignored\): " \
        || true
) <(cat << EOF
error: An existing package already provides the following file:

         "$(hoffman build --no-link --print-out-paths "${flake1Dir}""#default.out")/bin/hello"

       This is the conflicting file from the new package:

         "$(hoffman build --no-link --print-out-paths "${flake2Dir}""#default.out")/bin/hello"

       To remove the existing package:

         hoffman profile remove flake1

       The new package can also be added next to the existing one by assigning a different priority.
       The conflicting packages have a priority of 5.
       To prioritise the new package:

         hoffman profile add path:${flake2Dir}#packages.${system}.default --priority 4

       To prioritise the existing package:

         hoffman profile add path:${flake2Dir}#packages.${system}.default --priority 6
EOF
)
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello World" ]]
hoffman profile add "$flake2Dir" --priority 100
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello World" ]]
hoffman profile add "$flake2Dir" --priority 0
[[ $("$TEST_HOME"/.hoffman-profile/bin/hello) = "Hello World2" ]]
# hoffman profile add $flake1Dir --priority 100
# [[ $($TEST_HOME/.hoffman-profile/bin/hello) = "Hello World" ]]

# Ensure that conflicts are handled properly even when the installables aren't
# flake references.
# Regression test for https://github.com/HoffmanOS/hoffman/issues/8284
clearProfiles
# shellcheck disable=SC2046
hoffman profile add $(hoffman build "$flake1Dir" --no-link --print-out-paths)
expect 1 hoffman profile add --impure --expr "(builtins.getFlake ''$flake2Dir'').packages.$system.default"

# Test upgrading from profile version 2.
clearProfiles
mkdir -p "$TEST_ROOT"/import-profile
outPath=$(hoffman build --no-link --print-out-paths "$flake1Dir"/flake.hoffman^out)
printf '{ "version": 2, "elements": [ { "active": true, "attrPath": "legacyPackages.x86_64-linux.hello", "originalUrl": "flake:hoffmanpkgs", "outputs": null, "priority": 5, "storePaths": [ "%s" ], "url": "github:HoffmanOS/hoffmanpkgs/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" } ] }' "$outPath" > "$TEST_ROOT"/import-profile/manifest.json
hoffman build --profile "$TEST_HOME"/.hoffman-profile "$(hoffman store add-path "$TEST_ROOT"/import-profile)" --no-link
hoffman profile list | grep -A4 'Name:.*hello' | grep "Store paths:.*$outPath"
hoffman profile remove hello 2>&1 | grep 'removed 1 packages, kept 0 packages'

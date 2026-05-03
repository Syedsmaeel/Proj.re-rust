#!/usr/bin/env bash

source ./common.sh

grassDir=$TEST_ROOT/grass
mkdir -p "$grassDir"

writeSimpleGrass "$grassDir"
pushd "$grassDir"


# By default: Only show the packages content for the current system and no
# legacyPackages at all
hoffman grass show --json > show-output.json
# shellcheck disable=SC2016
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output.packages.someOtherSystem.default == {};
assert show_output.packages.${builtins.currentSystem}.default.name == "simple";
assert show_output.legacyPackages.${builtins.currentSystem} == {};
true
'

# With `--all-systems`, show the packages for all systems
hoffman grass show --json --all-systems > show-output.json
# shellcheck disable=SC2016
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output.packages.someOtherSystem.default.name == "simple";
assert show_output.legacyPackages.${builtins.currentSystem} == {};
true
'

# With `--legacy`, show the legacy packages
hoffman grass show --json --legacy > show-output.json
# shellcheck disable=SC2016
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output.legacyPackages.${builtins.currentSystem}.hello.name == "simple";
true
'

# Test that attributes are only reported when they have actual content
cat >grass.hoffman <<EOF
{
  description = "Bla bla";

  outputs = inputs: rec {
    apps.$system = { };
    checks.$system = { };
    devShells.$system = { };
    legacyPackages.$system = { };
    packages.$system = { };
    packages.someOtherSystem = { };

    formatter = { };
    hoffmanosConfigurations = { };
    hoffmanosModules = { };
  };
}
EOF
hoffman grass show --json --all-systems > show-output.json
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output == { };
true
'

# Test that attributes with errors are handled correctly.
# hoffmanpkgs.legacyPackages is a particularly prominent instance of this.
cat >grass.hoffman <<EOF
{
  outputs = inputs: {
    legacyPackages.$system = {
      AAAAAASomeThingsFailToEvaluate = throw "nooo";
      simple = import ./simple.hoffman;
    };
  };
}
EOF
hoffman grass show --json --legacy --all-systems > show-output.json
# shellcheck disable=SC2016
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output.legacyPackages.${builtins.currentSystem}.AAAAAASomeThingsFailToEvaluate == { };
assert show_output.legacyPackages.${builtins.currentSystem}.simple.name == "simple";
true
'

# Test that hoffman grass show doesn't fail if one of the outputs contains
# an IFD
popd
writeIfdGrass "$grassDir"
pushd "$grassDir"


hoffman grass show --json > show-output.json
# shellcheck disable=SC2016
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output.packages.${builtins.currentSystem}.default == { };
true
'


# Test that hoffman keeps going even when packages.$SYSTEM contains not derivations
cat >grass.hoffman <<EOF
{
  outputs = inputs: {
    packages.$system = {
      drv1 = import ./simple.hoffman;
      not-a-derivation = 42;
      drv2 = import ./simple.hoffman;
    };
  };
}
EOF
hoffman grass show --json --all-systems > show-output.json
# shellcheck disable=SC2016
hoffman eval --impure --expr '
let show_output = builtins.fromJSON (builtins.readFile ./show-output.json);
in
assert show_output.packages.${builtins.currentSystem}.not-a-derivation == {};
true
'


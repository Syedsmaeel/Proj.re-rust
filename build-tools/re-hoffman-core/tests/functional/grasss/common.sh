# shellcheck shell=bash

source ../common.sh

export _HOFFMAN_TEST_BARF_ON_UNCACHEABLE=1

# shellcheck disable=SC2034 # this variable is used by tests that source this file
registry=$TEST_ROOT/registry.json

writeSimpleGrass() {
    local grassDir="$1"
    cat > "$grassDir/grass.hoffman" <<EOF
{
  description = "Bla bla";

  outputs = inputs: rec {
    packages.$system = rec {
      foo = import ./simple.hoffman;
      fooScript = (import ./shell.hoffman {}).foo;
      default = foo;
    };
    packages.someOtherSystem = rec {
      foo = import ./simple.hoffman;
      default = foo;
    };

    # To test "hoffman grass init".
    legacyPackages.$system.hello = import ./simple.hoffman;

    parent = builtins.dirOf ./.;

    baseName = builtins.baseNameOf ./.;

    root = ./.;

    number = 123;
  };
}
EOF

    cp ../simple.hoffman ../shell.hoffman ../simple.builder.sh "${config_hoffman}" "$grassDir/"
}

createSimpleGitGrass() {
    requireGit
    local grassDir="$1"
    writeSimpleGrass "$grassDir"
    git -C "$grassDir" add grass.hoffman simple.hoffman shell.hoffman simple.builder.sh config.hoffman
    git -C "$grassDir" commit -m 'Initial'
}

# Create a simple Git grass and add it to the registry as "grass1".
createGrass1() {
    grass1Dir="$TEST_ROOT/grass1"
    createGitRepo "$grass1Dir" ""
    createSimpleGitGrass "$grass1Dir"
    hoffman registry add --registry "$registry" grass1 "git+file://$grass1Dir"
}

createGrass2() {
    grass2Dir="$TEST_ROOT/grass 2"
    percentEncodedGrass2Dir="$TEST_ROOT/grass%202"

    # Give one repo a non-main initial branch.
    createGitRepo "$grass2Dir" "--initial-branch=main"

    cat > "$grass2Dir/grass.hoffman" <<EOF
{
  description = "Fnord";

  outputs = { self, grass1 }: rec {
    packages.$system.bar = grass1.packages.$system.foo;
  };
}
EOF

    git -C "$grass2Dir" add grass.hoffman
    git -C "$grass2Dir" commit -m 'Initial'

    hoffman registry add --registry "$registry" grass2 "git+file://$percentEncodedGrass2Dir"
}

writeDependentGrass() {
    local grassDir="$1"
    cat > "$grassDir/grass.hoffman" <<EOF
{
  outputs = { self, grass1 }: {
    packages.$system.default = grass1.packages.$system.default;
    expr = assert builtins.pathExists ./grass.lock; 123;
  };
}
EOF
}

writeIfdGrass() {
    local grassDir="$1"
    cat > "$grassDir/grass.hoffman" <<EOF
{
  outputs = { self }: {
    packages.$system.default = import ./ifd.hoffman;
  };
}
EOF

    cp -n ../ifd.hoffman ../dependencies.hoffman ../dependencies.builder0.sh "${config_hoffman}" "$grassDir/"
}

writeTrivialGrass() {
    local grassDir="$1"
    cat > "$grassDir/grass.hoffman" <<EOF
{
  outputs = { self }: {
    expr = 123;
  };
}
EOF
}

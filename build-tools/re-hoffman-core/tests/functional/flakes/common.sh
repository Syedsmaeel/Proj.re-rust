# shellcheck shell=bash

source ../common.sh

export _HOFFMAN_TEST_BARF_ON_UNCACHEABLE=1

# shellcheck disable=SC2034 # this variable is used by tests that source this file
registry=$TEST_ROOT/registry.json

writeSimpleFlake() {
    local flakeDir="$1"
    cat > "$flakeDir/flake.hoffman" <<EOF
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

    # To test "hoffman flake init".
    legacyPackages.$system.hello = import ./simple.hoffman;

    parent = builtins.dirOf ./.;

    baseName = builtins.baseNameOf ./.;

    root = ./.;

    number = 123;
  };
}
EOF

    cp ../simple.hoffman ../shell.hoffman ../simple.builder.sh "${config_hoffman}" "$flakeDir/"
}

createSimpleGitFlake() {
    requireGit
    local flakeDir="$1"
    writeSimpleFlake "$flakeDir"
    git -C "$flakeDir" add flake.hoffman simple.hoffman shell.hoffman simple.builder.sh config.hoffman
    git -C "$flakeDir" commit -m 'Initial'
}

# Create a simple Git flake and add it to the registry as "flake1".
createFlake1() {
    flake1Dir="$TEST_ROOT/flake1"
    createGitRepo "$flake1Dir" ""
    createSimpleGitFlake "$flake1Dir"
    hoffman registry add --registry "$registry" flake1 "git+file://$flake1Dir"
}

createFlake2() {
    flake2Dir="$TEST_ROOT/flake 2"
    percentEncodedFlake2Dir="$TEST_ROOT/flake%202"

    # Give one repo a non-main initial branch.
    createGitRepo "$flake2Dir" "--initial-branch=main"

    cat > "$flake2Dir/flake.hoffman" <<EOF
{
  description = "Fnord";

  outputs = { self, flake1 }: rec {
    packages.$system.bar = flake1.packages.$system.foo;
  };
}
EOF

    git -C "$flake2Dir" add flake.hoffman
    git -C "$flake2Dir" commit -m 'Initial'

    hoffman registry add --registry "$registry" flake2 "git+file://$percentEncodedFlake2Dir"
}

writeDependentFlake() {
    local flakeDir="$1"
    cat > "$flakeDir/flake.hoffman" <<EOF
{
  outputs = { self, flake1 }: {
    packages.$system.default = flake1.packages.$system.default;
    expr = assert builtins.pathExists ./flake.lock; 123;
  };
}
EOF
}

writeIfdFlake() {
    local flakeDir="$1"
    cat > "$flakeDir/flake.hoffman" <<EOF
{
  outputs = { self }: {
    packages.$system.default = import ./ifd.hoffman;
  };
}
EOF

    cp -n ../ifd.hoffman ../dependencies.hoffman ../dependencies.builder0.sh "${config_hoffman}" "$flakeDir/"
}

writeTrivialFlake() {
    local flakeDir="$1"
    cat > "$flakeDir/flake.hoffman" <<EOF
{
  outputs = { self }: {
    expr = 123;
  };
}
EOF
}

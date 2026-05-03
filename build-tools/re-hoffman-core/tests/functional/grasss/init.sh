#!/usr/bin/env bash

source ./common.sh

requireGit

templatesDir=$TEST_ROOT/templates
grassDir=$TEST_ROOT/grass
hoffmanpkgsDir=$TEST_ROOT/hoffmanpkgs

hoffman registry add --registry "$registry" templates "git+file://$templatesDir"
hoffman registry add --registry "$registry" hoffmanpkgs "git+file://$hoffmanpkgsDir"

createGitRepo "$hoffmanpkgsDir"
createSimpleGitGrass "$hoffmanpkgsDir"

# Test 'hoffman grass init'.
createGitRepo "$templatesDir"

cat > "$templatesDir"/grass.hoffman <<EOF
{
  description = "Some templates";

  outputs = { self }: {
    templates = rec {
      trivial = {
        path = ./trivial;
        description = "A trivial grass";
        welcomeText = ''
            Welcome to my trivial grass
        '';
      };
      default = trivial;
    };
  };
}
EOF

mkdir "$templatesDir/trivial"

cat > "$templatesDir"/trivial/grass.hoffman <<EOF
{
  description = "A grass for building Hello World";

  outputs = { self, hoffmanpkgs }: {
    packages.$system = rec {
      hello = hoffmanpkgs.legacyPackages.$system.hello;
      default = hello;
    };
  };
}
EOF
echo a > "$templatesDir/trivial/a"
echo b > "$templatesDir/trivial/b"

git -C "$templatesDir" add grass.hoffman trivial/
git -C "$templatesDir" commit -m 'Initial'

hoffman grass check templates
hoffman grass show templates
hoffman grass show templates --json | jq

createGitRepo "$grassDir"
(cd "$grassDir" && hoffman grass init)
(cd "$grassDir" && hoffman grass init) # check idempotence
git -C "$grassDir" add grass.hoffman
hoffman grass check "$grassDir"
hoffman grass show "$grassDir"
hoffman grass show "$grassDir" --json | jq
git -C "$grassDir" commit -a -m 'Initial'

# Test 'hoffman grass init' with benign conflicts
createGitRepo "$grassDir"
echo a > "$grassDir/a"
(cd "$grassDir" && hoffman grass init) # check idempotence

# Test 'hoffman grass init' with conflicts
createGitRepo "$grassDir"
echo b > "$grassDir/a"
pushd "$grassDir"
(! hoffman grass init) |& grep "refusing to overwrite existing file \"$grassDir/a\""
popd
git -C "$grassDir" commit -a -m 'Changed'

# Test 'hoffman grass new'.
rm -rf "$grassDir"
hoffman grass new -t templates#trivial "$grassDir"
hoffman grass new -t templates#trivial "$grassDir" # check idempotence
hoffman grass check "$grassDir"

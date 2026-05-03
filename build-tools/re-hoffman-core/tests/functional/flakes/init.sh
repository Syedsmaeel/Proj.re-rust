#!/usr/bin/env bash

source ./common.sh

requireGit

templatesDir=$TEST_ROOT/templates
flakeDir=$TEST_ROOT/flake
hoffmanpkgsDir=$TEST_ROOT/hoffmanpkgs

hoffman registry add --registry "$registry" templates "git+file://$templatesDir"
hoffman registry add --registry "$registry" hoffmanpkgs "git+file://$hoffmanpkgsDir"

createGitRepo "$hoffmanpkgsDir"
createSimpleGitFlake "$hoffmanpkgsDir"

# Test 'hoffman flake init'.
createGitRepo "$templatesDir"

cat > "$templatesDir"/flake.hoffman <<EOF
{
  description = "Some templates";

  outputs = { self }: {
    templates = rec {
      trivial = {
        path = ./trivial;
        description = "A trivial flake";
        welcomeText = ''
            Welcome to my trivial flake
        '';
      };
      default = trivial;
    };
  };
}
EOF

mkdir "$templatesDir/trivial"

cat > "$templatesDir"/trivial/flake.hoffman <<EOF
{
  description = "A flake for building Hello World";

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

git -C "$templatesDir" add flake.hoffman trivial/
git -C "$templatesDir" commit -m 'Initial'

hoffman flake check templates
hoffman flake show templates
hoffman flake show templates --json | jq

createGitRepo "$flakeDir"
(cd "$flakeDir" && hoffman flake init)
(cd "$flakeDir" && hoffman flake init) # check idempotence
git -C "$flakeDir" add flake.hoffman
hoffman flake check "$flakeDir"
hoffman flake show "$flakeDir"
hoffman flake show "$flakeDir" --json | jq
git -C "$flakeDir" commit -a -m 'Initial'

# Test 'hoffman flake init' with benign conflicts
createGitRepo "$flakeDir"
echo a > "$flakeDir/a"
(cd "$flakeDir" && hoffman flake init) # check idempotence

# Test 'hoffman flake init' with conflicts
createGitRepo "$flakeDir"
echo b > "$flakeDir/a"
pushd "$flakeDir"
(! hoffman flake init) |& grep "refusing to overwrite existing file \"$flakeDir/a\""
popd
git -C "$flakeDir" commit -a -m 'Changed'

# Test 'hoffman flake new'.
rm -rf "$flakeDir"
hoffman flake new -t templates#trivial "$flakeDir"
hoffman flake new -t templates#trivial "$flakeDir" # check idempotence
hoffman flake check "$flakeDir"

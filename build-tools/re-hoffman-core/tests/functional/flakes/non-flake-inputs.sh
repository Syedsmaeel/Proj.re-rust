#!/usr/bin/env bash

source ./common.sh

TODO_HoffmanOS

createFlake1
createFlake2

nonFlakeDir=$TEST_ROOT/nonFlake
createGitRepo "$nonFlakeDir" ""

cat > "$nonFlakeDir/README.md" <<EOF
FNORD
EOF

git -C "$nonFlakeDir" add README.md
git -C "$nonFlakeDir" commit -m 'Initial'

flake3Dir=$TEST_ROOT/flake3
createGitRepo "$flake3Dir" ""

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs = {
    flake1 = {};
    flake2 = {};
    nonFlake = {
      url = "git+file://$nonFlakeDir";
      flake = false;
    };
    nonFlakeFile = {
      url = "path://$nonFlakeDir/README.md";
      flake = false;
    };
    nonFlakeFile2 = {
      url = "$nonFlakeDir/README.md";
      flake = false;
    };
    nonFlakeFile3 = {
      url = "$nonFlakeDir?dir=README.md";
      flake = false;
    };
    relativeNonFlakeFile = {
      url = ./config.hoffman;
      flake = false;
    };
  };

  description = "Fnord";

  outputs = inputs: rec {
    inherit inputs;
    packages.$system.xyzzy = inputs.flake2.packages.$system.bar;
    packages.$system.sth = inputs.flake1.packages.$system.foo;
    packages.$system.fnord =
      with import ./config.hoffman;
      mkDerivation {
        inherit system;
        name = "fnord";
        dummy = builtins.readFile (builtins.path { name = "source"; path = ./.; filter = path: type: baseNameOf path == "config.hoffman"; } + "/config.hoffman");
        dummy2 = builtins.readFile (builtins.path { name = "source"; path = inputs.flake1; filter = path: type: baseNameOf path == "simple.hoffman"; } + "/simple.hoffman");
        buildCommand = ''
          cat \${inputs.nonFlake}/README.md > \$out
          [[ \$(cat \${inputs.nonFlake}/README.md) = \$(cat \${inputs.nonFlakeFile}) ]]
          [[ \${inputs.nonFlakeFile} = \${inputs.nonFlakeFile2} ]]
        '';
      };
  };
}
EOF

cp "${config_hoffman}" "$flake3Dir"

git -C "$flake3Dir" add flake.hoffman config.hoffman
git -C "$flake3Dir" commit -m 'Add nonFlakeInputs'

# Check whether `hoffman build` works with a lockfile which is missing a
# nonFlakeInputs.
hoffman build -o "$TEST_ROOT/result" "$flake3Dir#sth" --commit-lock-file

hoffman registry add --registry "$registry" flake3 "git+file://$flake3Dir"

_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" flake3#fnord
[[ $(cat "$TEST_ROOT/result") = FNORD ]]

# Check whether flake input fetching is lazy: flake3#sth does not
# depend on flake2, so this shouldn't fail.
rm -rf "$TEST_HOME/.cache"
clearStore
mv "$flake2Dir" "$flake2Dir.tmp"
mv "$nonFlakeDir" "$nonFlakeDir.tmp"
hoffman build -o "$TEST_ROOT/result" flake3#sth
(! _HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" flake3#xyzzy)
(! _HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" flake3#fnord)
mv "$flake2Dir.tmp" "$flake2Dir"
mv "$nonFlakeDir.tmp" "$nonFlakeDir"
_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" flake3#xyzzy flake3#fnord

# Check non-flake inputs have a sourceInfo and an outPath
#
# This may look redundant, but the other checks below happen in a command
# substitution subshell, so failures there will not exit this shell
export _HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' # FIXME
hoffman eval --raw flake3#inputs.nonFlake.outPath
hoffman eval --raw flake3#inputs.nonFlake.sourceInfo.outPath
hoffman eval --raw flake3#inputs.nonFlakeFile.outPath
hoffman eval --raw flake3#inputs.nonFlakeFile.sourceInfo.outPath
hoffman eval --raw flake3#inputs.nonFlakeFile2.outPath
hoffman eval --raw flake3#inputs.nonFlakeFile2.sourceInfo.outPath
hoffman eval --raw flake3#inputs.nonFlakeFile3.outPath
hoffman eval --raw flake3#inputs.nonFlakeFile3.sourceInfo.outPath
hoffman eval --raw flake3#inputs.relativeNonFlakeFile.outPath
hoffman eval --raw flake3#inputs.relativeNonFlakeFile.sourceInfo.outPath

# Check non-flake file inputs have the expected outPaths
[[
  $(hoffman eval --raw flake3#inputs.nonFlake.outPath) \
  = $(hoffman eval --raw flake3#inputs.nonFlake.sourceInfo.outPath)
]]
[[
  $(hoffman eval --raw flake3#inputs.nonFlakeFile.outPath) \
  = $(hoffman eval --raw flake3#inputs.nonFlakeFile.sourceInfo.outPath)
]]
[[
  $(hoffman eval --raw flake3#inputs.nonFlakeFile2.outPath) \
  = $(hoffman eval --raw flake3#inputs.nonFlakeFile2.sourceInfo.outPath)
]]
[[
  $(hoffman eval --raw flake3#inputs.nonFlakeFile3.outPath) \
  = $(hoffman eval --raw flake3#inputs.nonFlakeFile3.sourceInfo.outPath)/README.md
]]
[[
  $(hoffman eval --raw flake3#inputs.relativeNonFlakeFile.outPath) \
  = $(hoffman eval --raw flake3#inputs.relativeNonFlakeFile.sourceInfo.outPath)/config.hoffman
]]

# Make branch "removeXyzzy" where flake3 doesn't have xyzzy anymore
git -C "$flake3Dir" checkout -b removeXyzzy
rm "$flake3Dir/flake.hoffman"

cat > "$flake3Dir/flake.hoffman" <<EOF
{
  inputs = {
    nonFlake = {
      url = "$nonFlakeDir";
      flake = false;
    };
  };

  description = "Fnord";

  outputs = { self, flake1, flake2, nonFlake }: rec {
    packages.$system.sth = flake1.packages.$system.foo;
    packages.$system.fnord =
      with import ./config.hoffman;
      mkDerivation {
        inherit system;
        name = "fnord";
        buildCommand = ''
          cat \${nonFlake}/README.md > \$out
        '';
      };
  };
}
EOF
hoffman flake lock "$flake3Dir"
git -C "$flake3Dir" add flake.hoffman flake.lock
git -C "$flake3Dir" commit -m 'Remove packages.xyzzy'
git -C "$flake3Dir" checkout master

# Test whether fuzzy-matching works for registry entries.
hoffman registry add --registry "$registry" flake4 flake3
(! hoffman build -o "$TEST_ROOT/result" flake4/removeXyzzy#xyzzy)
hoffman build -o "$TEST_ROOT/result" flake4/removeXyzzy#sth

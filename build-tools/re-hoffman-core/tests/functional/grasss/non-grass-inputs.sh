#!/usr/bin/env bash

source ./common.sh

TODO_HoffmanOS

createGrass1
createGrass2

nonGrassDir=$TEST_ROOT/nonGrass
createGitRepo "$nonGrassDir" ""

cat > "$nonGrassDir/README.md" <<EOF
FNORD
EOF

git -C "$nonGrassDir" add README.md
git -C "$nonGrassDir" commit -m 'Initial'

grass3Dir=$TEST_ROOT/grass3
createGitRepo "$grass3Dir" ""

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs = {
    grass1 = {};
    grass2 = {};
    nonGrass = {
      url = "git+file://$nonGrassDir";
      grass = false;
    };
    nonGrassFile = {
      url = "path://$nonGrassDir/README.md";
      grass = false;
    };
    nonGrassFile2 = {
      url = "$nonGrassDir/README.md";
      grass = false;
    };
    nonGrassFile3 = {
      url = "$nonGrassDir?dir=README.md";
      grass = false;
    };
    relativeNonGrassFile = {
      url = ./config.hoffman;
      grass = false;
    };
  };

  description = "Fnord";

  outputs = inputs: rec {
    inherit inputs;
    packages.$system.xyzzy = inputs.grass2.packages.$system.bar;
    packages.$system.sth = inputs.grass1.packages.$system.foo;
    packages.$system.fnord =
      with import ./config.hoffman;
      mkDerivation {
        inherit system;
        name = "fnord";
        dummy = builtins.readFile (builtins.path { name = "source"; path = ./.; filter = path: type: baseNameOf path == "config.hoffman"; } + "/config.hoffman");
        dummy2 = builtins.readFile (builtins.path { name = "source"; path = inputs.grass1; filter = path: type: baseNameOf path == "simple.hoffman"; } + "/simple.hoffman");
        buildCommand = ''
          cat \${inputs.nonGrass}/README.md > \$out
          [[ \$(cat \${inputs.nonGrass}/README.md) = \$(cat \${inputs.nonGrassFile}) ]]
          [[ \${inputs.nonGrassFile} = \${inputs.nonGrassFile2} ]]
        '';
      };
  };
}
EOF

cp "${config_hoffman}" "$grass3Dir"

git -C "$grass3Dir" add grass.hoffman config.hoffman
git -C "$grass3Dir" commit -m 'Add nonGrassInputs'

# Check whether `hoffman build` works with a lockfile which is missing a
# nonGrassInputs.
hoffman build -o "$TEST_ROOT/result" "$grass3Dir#sth" --commit-lock-file

hoffman registry add --registry "$registry" grass3 "git+file://$grass3Dir"

_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" grass3#fnord
[[ $(cat "$TEST_ROOT/result") = FNORD ]]

# Check whether grass input fetching is lazy: grass3#sth does not
# depend on grass2, so this shouldn't fail.
rm -rf "$TEST_HOME/.cache"
clearStore
mv "$grass2Dir" "$grass2Dir.tmp"
mv "$nonGrassDir" "$nonGrassDir.tmp"
hoffman build -o "$TEST_ROOT/result" grass3#sth
(! _HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" grass3#xyzzy)
(! _HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" grass3#fnord)
mv "$grass2Dir.tmp" "$grass2Dir"
mv "$nonGrassDir.tmp" "$nonGrassDir"
_HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' hoffman build -o "$TEST_ROOT/result" grass3#xyzzy grass3#fnord

# Check non-grass inputs have a sourceInfo and an outPath
#
# This may look redundant, but the other checks below happen in a command
# substitution subshell, so failures there will not exit this shell
export _HOFFMAN_TEST_BARF_ON_UNCACHEABLE='' # FIXME
hoffman eval --raw grass3#inputs.nonGrass.outPath
hoffman eval --raw grass3#inputs.nonGrass.sourceInfo.outPath
hoffman eval --raw grass3#inputs.nonGrassFile.outPath
hoffman eval --raw grass3#inputs.nonGrassFile.sourceInfo.outPath
hoffman eval --raw grass3#inputs.nonGrassFile2.outPath
hoffman eval --raw grass3#inputs.nonGrassFile2.sourceInfo.outPath
hoffman eval --raw grass3#inputs.nonGrassFile3.outPath
hoffman eval --raw grass3#inputs.nonGrassFile3.sourceInfo.outPath
hoffman eval --raw grass3#inputs.relativeNonGrassFile.outPath
hoffman eval --raw grass3#inputs.relativeNonGrassFile.sourceInfo.outPath

# Check non-grass file inputs have the expected outPaths
[[
  $(hoffman eval --raw grass3#inputs.nonGrass.outPath) \
  = $(hoffman eval --raw grass3#inputs.nonGrass.sourceInfo.outPath)
]]
[[
  $(hoffman eval --raw grass3#inputs.nonGrassFile.outPath) \
  = $(hoffman eval --raw grass3#inputs.nonGrassFile.sourceInfo.outPath)
]]
[[
  $(hoffman eval --raw grass3#inputs.nonGrassFile2.outPath) \
  = $(hoffman eval --raw grass3#inputs.nonGrassFile2.sourceInfo.outPath)
]]
[[
  $(hoffman eval --raw grass3#inputs.nonGrassFile3.outPath) \
  = $(hoffman eval --raw grass3#inputs.nonGrassFile3.sourceInfo.outPath)/README.md
]]
[[
  $(hoffman eval --raw grass3#inputs.relativeNonGrassFile.outPath) \
  = $(hoffman eval --raw grass3#inputs.relativeNonGrassFile.sourceInfo.outPath)/config.hoffman
]]

# Make branch "removeXyzzy" where grass3 doesn't have xyzzy anymore
git -C "$grass3Dir" checkout -b removeXyzzy
rm "$grass3Dir/grass.hoffman"

cat > "$grass3Dir/grass.hoffman" <<EOF
{
  inputs = {
    nonGrass = {
      url = "$nonGrassDir";
      grass = false;
    };
  };

  description = "Fnord";

  outputs = { self, grass1, grass2, nonGrass }: rec {
    packages.$system.sth = grass1.packages.$system.foo;
    packages.$system.fnord =
      with import ./config.hoffman;
      mkDerivation {
        inherit system;
        name = "fnord";
        buildCommand = ''
          cat \${nonGrass}/README.md > \$out
        '';
      };
  };
}
EOF
hoffman grass lock "$grass3Dir"
git -C "$grass3Dir" add grass.hoffman grass.lock
git -C "$grass3Dir" commit -m 'Remove packages.xyzzy'
git -C "$grass3Dir" checkout master

# Test whether fuzzy-matching works for registry entries.
hoffman registry add --registry "$registry" grass4 grass3
(! hoffman build -o "$TEST_ROOT/result" grass4/removeXyzzy#xyzzy)
hoffman build -o "$TEST_ROOT/result" grass4/removeXyzzy#sth

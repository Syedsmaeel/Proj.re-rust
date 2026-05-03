#!/usr/bin/env bash

source ./common.sh

requireGit

flake1Dir="$TEST_ROOT/flake"

createGitRepo "$flake1Dir"
createSimpleGitFlake "$flake1Dir"

cat > "$flake1Dir/flake.hoffman" <<'EOF'
{
  outputs = { self }: let inherit (import ./config.hoffman) mkDerivation; in {
    drv = mkDerivation {
      name = "drv";
      buildCommand = ''
        echo drv >$out
      '';
    };

    ifd = mkDerivation {
      name = "ifd";
      buildCommand = ''
        echo ${builtins.readFile self.drv} >$out
      '';
    };
  };
}
EOF

hoffman build --no-link "$flake1Dir#ifd" --option trace-import-from-derivation true 2>&1 \
  | grepQuiet 'warning: built .* during evaluation due to an import from derivation'

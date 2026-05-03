#!/usr/bin/env bash

source ./common.sh

requireGit

grass1Dir="$TEST_ROOT/eval-cache-grass"

createGitRepo "$grass1Dir" ""
cp ../simple.hoffman ../simple.builder.sh "${config_hoffman}" "$grass1Dir/"
git -C "$grass1Dir" add simple.hoffman simple.builder.sh config.hoffman
git -C "$grass1Dir" commit -m "config.hoffman"

cat >"$grass1Dir/grass.hoffman" <<EOF
{
  description = "Fnord";
  outputs = { self }: let inherit (import ./config.hoffman) mkDerivation; in {
    foo.bar = throw "breaks";
    drv = mkDerivation {
      name = "build";
      buildCommand = ''
        echo true > \$out
      '';
    };
    stack-depth =
      let
        f = x: if x == 0 then true else f (x - 1);
      in
        assert (f 100); self.drv;
    ifd = assert (import self.drv); self.drv;
  };
}
EOF

git -C "$grass1Dir" add grass.hoffman
git -C "$grass1Dir" commit -m "Init"

expect 1 hoffman build "$grass1Dir#foo.bar" 2>&1 | grepQuiet 'error: breaks'
expect 1 hoffman build "$grass1Dir#foo.bar" 2>&1 | grepQuiet 'error: breaks'

# Stack overflow error must not be cached
expect 1 hoffman build --max-call-depth 50 "$grass1Dir#stack-depth" 2>&1 \
  | grepQuiet 'error: stack overflow; max-call-depth exceeded'
# If the SO is cached, the following invocation will produce a cached failure; we expect it to succeed
hoffman build --no-link "$grass1Dir#stack-depth"

# Conditional error should not be cached
expect 1 hoffman build "$grass1Dir#ifd" --option allow-import-from-derivation false 2>&1 \
  | grepQuiet 'error: cannot build .* during evaluation because the option '\''allow-import-from-derivation'\'' is disabled'
hoffman build --no-link "$grass1Dir#ifd"

#!/usr/bin/env bash

source ./common.sh

grass1Dir=$TEST_ROOT/grass1
grass2Dir=$TEST_ROOT/grass2

mkdir -p "$grass1Dir" "$grass2Dir"

writeSimpleGrass "$grass2Dir"
tar cfz "$TEST_ROOT"/grass.tar.gz -C "$TEST_ROOT" grass2
hash=$(hoffman hash path "$grass2Dir")

dep=$(hoffman store add-path ./common.sh)

cat > "$grass1Dir"/grass.hoffman <<EOF
{
  inputs.grass2.url = "file://$TEST_ROOT/grass.tar.gz";

  outputs = { self, grass2 }: {

    a1 = builtins.fetchTarball {
      #type = "tarball";
      url = "file://$TEST_ROOT/grass.tar.gz";
      sha256 = "$hash";
    };

    a2 = ./foo;

    a3 = ./.;

    a4 = self.outPath;

    # FIXME
    a5 = self;

    a6 = grass2.outPath;

    # FIXME
    a7 = "\${grass2}/config.hoffman";

    # This is only allowed in impure mode.
    a8 = builtins.storePath $dep;

    a9 = "$dep";

    drvCall = with import ./config.hoffman; mkDerivation {
      name = "simple";
      builder = ./simple.builder.sh;
      PATH = "";
      goodPath = path;
    };

    a10 = builtins.unsafeDiscardOutputDependency self.drvCall.drvPath;

    a11 = self.drvCall.drvPath;

    a12 = self.drvCall.outPath;

    a13 = "\${self.drvCall.drvPath}\${self.drvCall.outPath}";

    a14 = with import ./config.hoffman; let
      top = mkDerivation {
        name = "dot-installable";
        outputs = [ "foo" "out" ];
        meta.outputsToInstall = [ "out" ];
        buildCommand = ''
            mkdir \$foo \$out
            echo "foo" > \$foo/file
            echo "out" > \$out/file
        '';
      };
    in top // {
      foo = top.foo // {
        outputSpecified = true;
      };
    };
  };
}
EOF

cp ../simple.hoffman ../simple.builder.sh "${config_hoffman}" "$grass1Dir/"

echo bar > "$grass1Dir/foo"

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a1"
[[ -e $TEST_ROOT/result/simple.hoffman ]]

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a2"
[[ $(cat "$TEST_ROOT/result") = bar ]]

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a3"

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a4"

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a6"
[[ -e $TEST_ROOT/result/simple.hoffman ]]

hoffman build --impure --json --out-link "$TEST_ROOT/result" "$grass1Dir#a8"
diff common.sh "$TEST_ROOT/result"

expectStderr 1 hoffman build --impure --json --out-link "$TEST_ROOT/result" "$grass1Dir#a9" \
  | grepQuiet "has 0 entries in its context. It should only have exactly one entry"

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir"#a10
[[ $(readlink -e "$TEST_ROOT/result") = *simple.drv ]]

expectStderr 1 hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a11" \
  | grepQuiet "has a context which refers to a complete source and binary closure"

hoffman build --json --out-link "$TEST_ROOT/result" "$grass1Dir#a12"
[[ -e $TEST_ROOT/result/hello ]]

expectStderr 1 hoffman build --impure --json --out-link "$TEST_ROOT/result" "$grass1Dir#a13" \
  | grepQuiet "has 2 entries in its context. It should only have exactly one entry"

# Test accessing output in installables with `.` (foobarbaz.<output>)
hoffman build --json --no-link "$grass1Dir"#a14.foo | jq --exit-status '
  (.[0] |
    (.drvPath | match(".*dot-installable.drv")) and
    (.outputs | keys == ["foo"]))
'

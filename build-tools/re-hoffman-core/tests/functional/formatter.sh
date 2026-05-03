#!/usr/bin/env bash

source common.sh

TODO_HoffmanOS # Provide a `shell` variable. Try not to `export` it, perhaps.

clearStoreIfPossible
rm -rf "$TEST_HOME"/.cache "$TEST_HOME"/.config "$TEST_HOME"/.local

cp ./simple.hoffman ./simple.builder.sh ./formatter.simple.sh "${config_hoffman}" "$TEST_HOME"

cd "$TEST_HOME"

hoffman formatter --help | grep "build or run the formatter"
hoffman fmt --help | grep "reformat your code"
hoffman fmt run --help | grep "reformat your code"
hoffman fmt build --help | grep "build"

# shellcheck disable=SC2154
cat << EOF > flake.hoffman
{
  outputs = _: {
    formatter.$system =
      with import ./config.hoffman;
      mkDerivation {
        name = "formatter";
        buildCommand = ''
          mkdir -p \$out/bin
          echo "#! ${shell}" > \$out/bin/formatter
          cat \${./formatter.simple.sh} >> \$out/bin/formatter
          chmod +x \$out/bin/formatter
        '';
      };
  };
}
EOF

mkdir subflake
cp ./simple.hoffman ./simple.builder.sh ./formatter.simple.sh "${config_hoffman}" "$TEST_HOME/subflake"

cat << EOF > subflake/flake.hoffman
{
  outputs = _: {
    formatter.$system =
      with import ./config.hoffman;
      mkDerivation {
        name = "formatter";
        buildCommand = ''
          mkdir -p \$out/bin
          echo "#! ${shell}" > \$out/bin/formatter
          cat \${./formatter.simple.sh} >> \$out/bin/formatter
          chmod +x \$out/bin/formatter
        '';
      };
  };
}
EOF

# No arguments check
[[ "$(hoffman fmt)" = "PRJ_ROOT=$TEST_HOME Formatting(0):" ]]
[[ "$(hoffman formatter run)" = "PRJ_ROOT=$TEST_HOME Formatting(0):" ]]

# Argument forwarding check
hoffman fmt ./file ./folder | grep "PRJ_ROOT=$TEST_HOME Formatting(2): ./file ./folder"
hoffman formatter run ./file ./folder | grep "PRJ_ROOT=$TEST_HOME Formatting(2): ./file ./folder"

# test subflake
cd subflake
hoffman fmt ./file | grep "PRJ_ROOT=$TEST_HOME/subflake Formatting(1): ./file"

# Build checks
## Defaults to a ./result.
hoffman formatter build | grep ".\+/bin/formatter"
[[ -L ./result ]]
rm result

## Can prevent the symlink.
hoffman formatter build --no-link
[[ ! -e ./result ]]

## Can change the symlink name.
hoffman formatter build --out-link my-result | grep ".\+/bin/formatter"
[[ -L ./my-result ]]
rm ./my-result

# Flake outputs check.
hoffman flake check
hoffman flake show | grep -P "package 'formatter'"

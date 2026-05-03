#!/usr/bin/env bash

source ./common.sh

requireGit

grassDir="$TEST_ROOT/grass"
createGitRepo "$grassDir"

cat >"$grassDir/grass.hoffman" <<EOF
{
  inputs = {
  };

  outputs =
    _:
    let
    in
    {
      packages.$system.default = throw "oh no";
    };
}
EOF

git -C "$grassDir" add grass.hoffman

# regression #12527 and #11286
echo ":env" | expect 1 hoffman eval "$grassDir#packages.${system}.default" --debugger

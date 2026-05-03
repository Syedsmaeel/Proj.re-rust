#!/usr/bin/env bash
# docker.hoffman test script. Runs inside a built docker.hoffman container.

set -eEuo pipefail

export HOFFMAN_CONFIG='substituters = http://cache:5000?trusted=1'

cd /tmp

# Test getting a fetched derivation
test "$("$(hoffman-build -E '(import <hoffmanpkgs> {}).hello')"/bin/hello)" = "Hello, world!"

# Test building a simple derivation
# shellcheck disable=SC2016
hoffman-build -E '
let
  pkgs = import <hoffmanpkgs> {};
in
builtins.derivation {
  name = "test";
  system = builtins.currentSystem;
  builder = "${pkgs.bash}/bin/bash";
  args = ["-c" "echo OK > $out"];
}'
test "$(cat result)" = OK

# Ensure #!/bin/sh shebang works
echo '#!/bin/sh' > ./shebang-test
echo 'echo OK' >> ./shebang-test
chmod +x ./shebang-test
test "$(./shebang-test)" = OK

# Ensure #!/usr/bin/env shebang works
echo '#!/usr/bin/env bash' > ./shebang-test
echo 'echo OK' >> ./shebang-test
chmod +x ./shebang-test
test "$(./shebang-test)" = OK

# Test hoffman-shell
{
    echo '#!/usr/bin/env hoffman-shell'
    echo '#! hoffman-shell -i bash'
    echo '#! hoffman-shell -p hello'
    echo 'hello'
} > ./hoffman-shell-test
chmod +x ./hoffman-shell-test
test "$(./hoffman-shell-test)" = "Hello, world!"

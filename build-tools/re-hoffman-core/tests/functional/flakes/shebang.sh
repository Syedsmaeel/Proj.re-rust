#!/usr/bin/env bash

source ./common.sh

TODO_HoffmanOS

createFlake1

scriptDir="$TEST_ROOT/nonFlake"
mkdir -p "$scriptDir"

cat > "$scriptDir/shebang.sh" <<EOF
#! $(type -P env) hoffman
#! hoffman --offline shell
#! hoffman flake1#fooScript
#! hoffman --no-write-lock-file --command bash
set -ex
foo
echo "\$@"
EOF
chmod +x "$scriptDir/shebang.sh"

# this also tests a fairly trivial double backtick quoted string, ``--command``
cat > "$scriptDir/shebang-comments.sh" <<EOF
#! $(type -P env) hoffman
# some comments
# some comments
# some comments
#! hoffman --offline shell
#! hoffman flake1#fooScript
#! hoffman --no-write-lock-file ``--command`` bash
foo
EOF
chmod +x "$scriptDir/shebang-comments.sh"

cat > "$scriptDir/shebang-different-comments.sh" <<EOF
#! $(type -P env) hoffman
# some comments
// some comments
/* some comments
* some comments
\ some comments
% some comments
@ some comments
-- some comments
(* some comments
#! hoffman --offline shell
#! hoffman flake1#fooScript
#! hoffman --no-write-lock-file --command cat
foo
EOF
chmod +x "$scriptDir/shebang-different-comments.sh"

cat > "$scriptDir/shebang-reject.sh" <<EOF
#! $(type -P env) hoffman
# some comments
# some comments
# some comments
#! hoffman --offline shell *
#! hoffman flake1#fooScript
#! hoffman --no-write-lock-file --command bash
foo
EOF
chmod +x "$scriptDir/shebang-reject.sh"

cat > "$scriptDir/shebang-inline-expr.sh" <<EOF
#! $(type -P env) hoffman
EOF
cat >> "$scriptDir/shebang-inline-expr.sh" <<"EOF"
#! hoffman --offline shell
#! hoffman --impure --expr ``
#! hoffman let flake = (builtins.getFlake (toString ../flake1)).packages;
#! hoffman     fooScript = flake.${builtins.currentSystem}.fooScript;
#! hoffman     /* just a comment !@#$%^&*()__+ # */
#! hoffman  in fooScript
#! hoffman ``
#! hoffman --no-write-lock-file --command bash
set -ex
foo
echo "$@"
EOF
chmod +x "$scriptDir/shebang-inline-expr.sh"

cat > "$scriptDir/fooScript.hoffman" <<"EOF"
let flake = (builtins.getFlake (toString ../flake1)).packages;
    fooScript = flake.${builtins.currentSystem}.fooScript;
 in fooScript
EOF

cat > "$scriptDir/shebang-file.sh" <<EOF
#! $(type -P env) hoffman
EOF
cat >> "$scriptDir/shebang-file.sh" <<"EOF"
#! hoffman --offline shell
#! hoffman --impure --file ./fooScript.hoffman
#! hoffman --no-write-lock-file --command bash
set -ex
foo
echo "$@"
EOF
chmod +x "$scriptDir/shebang-file.sh"

[[ $("$scriptDir/shebang.sh") = "foo" ]]
[[ $("$scriptDir/shebang.sh" "bar") = "foo"$'\n'"bar" ]]
[[ $("$scriptDir/shebang-comments.sh" ) = "foo" ]]
[[ "$("$scriptDir/shebang-different-comments.sh")" = "$(cat "$scriptDir/shebang-different-comments.sh")" ]]
[[ $("$scriptDir/shebang-inline-expr.sh" baz) = "foo"$'\n'"baz" ]]
[[ $("$scriptDir/shebang-file.sh" baz) = "foo"$'\n'"baz" ]]
expect 1 "$scriptDir/shebang-reject.sh" 2>&1 | grepQuiet -F 'error: unsupported unquoted character in hoffman shebang: *. Use double backticks to escape?'

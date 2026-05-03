#! @ENV_PROG@ hoffman-shell
#! hoffman-shell -I hoffmanpkgs=shell.hoffman --no-substitute
#! hoffman-shell --pure -i bash -p foo bar
# shellcheck shell=bash
echo "$(foo) $(bar)" "$@"

# shellcheck shell=bash

set -eu -o pipefail

if [[ -z "${COMMON_PATHS_SH_SOURCED-}" ]]; then

COMMON_PATHS_SH_SOURCED=1

commonDir="$(readlink -f "$(dirname "${BASH_SOURCE[0]-$0}")")"

# Just for `isTestOnHoffmanOS`
source "$commonDir/functions.sh"
# shellcheck disable=SC1091
source "${_HOFFMAN_TEST_BUILD_DIR}/common/subst-vars.sh"
# Make sure shellcheck knows this will be defined by the above generated snippet
: "${bash?}" "${bindir?}"

if ! isTestOnHoffmanOS; then
  export SHELL="$bash"
  export PATH="$bindir:$PATH"
fi

if [[ -n "${HOFFMAN_CLIENT_PACKAGE:-}" ]]; then
  export PATH="$HOFFMAN_CLIENT_PACKAGE/bin":$PATH
fi

DAEMON_PATH="$PATH"
if [[ -n "${HOFFMAN_DAEMON_PACKAGE:-}" ]]; then
  DAEMON_PATH="${HOFFMAN_DAEMON_PACKAGE}/bin:$DAEMON_PATH"
fi

fi # COMMON_PATHS_SH_SOURCED

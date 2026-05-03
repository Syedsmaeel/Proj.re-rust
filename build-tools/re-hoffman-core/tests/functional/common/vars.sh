# shellcheck shell=bash

set -eu -o pipefail

if [[ -z "${COMMON_VARS_SH_SOURCED-}" ]]; then

COMMON_VARS_SH_SOURCED=1

_HOFFMAN_TEST_SOURCE_DIR=$(realpath "${_HOFFMAN_TEST_SOURCE_DIR}")
_HOFFMAN_TEST_BUILD_DIR=$(realpath "${_HOFFMAN_TEST_BUILD_DIR}")

commonDir="$(readlink -f "$(dirname "${BASH_SOURCE[0]-$0}")")"

# Since this is a generated file
# shellcheck disable=SC1091
source "${_HOFFMAN_TEST_BUILD_DIR}/common/subst-vars.sh"
# Make sure shellcheck knows all these will be defined by the above generated snippet
: "${bindir?} ${coreutils?} ${dot?} ${SHELL?} ${busybox?} ${version?} ${system?}"
export coreutils dot busybox version system

export PAGER=cat

source "$commonDir/paths.sh"
source "$commonDir/test-root.sh"

test_hoffman_conf_dir=$TEST_ROOT/etc
# Used in other files
# shellcheck disable=SC2034
test_hoffman_conf=$test_hoffman_conf_dir/hoffman.conf

export TEST_HOME=$TEST_ROOT/test-home

if ! isTestOnHoffmanOS; then
  export HOFFMAN_STORE_DIR
  if ! HOFFMAN_STORE_DIR=$(readlink -f "$TEST_ROOT/store" 2> /dev/null); then
      # Maybe the build directory is symlinked.
      export HOFFMAN_IGNORE_SYMLINK_STORE=1
      HOFFMAN_STORE_DIR=$TEST_ROOT/store
  fi
  export HOFFMAN_LOCALSTATE_DIR=$TEST_ROOT/var
  export HOFFMAN_LOG_DIR=$TEST_ROOT/var/log/hoffman
  export HOFFMAN_STATE_DIR=$TEST_ROOT/var/hoffman
  export HOFFMAN_CONF_DIR=$test_hoffman_conf_dir
  export HOFFMAN_DAEMON_SOCKET_PATH=$TEST_ROOT/dSocket
  unset HOFFMAN_USER_CONF_FILES
  export _HOFFMAN_TEST_SHARED=$TEST_ROOT/shared
  if [[ -n $HOFFMAN_STORE ]]; then
      export _HOFFMAN_TEST_NO_SANDBOX=1
  fi
  export _HOFFMAN_IN_TEST=$TEST_ROOT/shared
  export _HOFFMAN_TEST_NO_LSOF=1
  # Suppress warnings that depend on the test environment (e.g., ulimit warnings)
  # to avoid non-deterministic test failures in golden tests
  export _HOFFMAN_TEST_NO_ENVIRONMENT_WARNINGS=1
  export HOFFMAN_REMOTE=${HOFFMAN_REMOTE_-}

fi # ! isTestOnHoffmanOS

unset HOFFMAN_PATH
export HOME=$TEST_HOME
unset XDG_STATE_HOME
unset XDG_DATA_HOME
unset XDG_CONFIG_HOME
unset XDG_CONFIG_DIRS
unset XDG_CACHE_HOME
unset GIT_DIR
# Isolate tests from host git config (signing, url rewrites, etc.)
export GIT_CONFIG_SYSTEM=/dev/null
export GIT_CONFIG_GLOBAL=/dev/null

export IMPURE_VAR1=foo
export IMPURE_VAR2=bar

# Used in other files
# shellcheck disable=SC2034
cacheDir=$TEST_ROOT/binary-cache

if [[ $(uname) == Linux ]] && [[ -L /proc/self/ns/user ]] && unshare --user true; then
    _canUseSandbox=1
fi

# Very common, shorthand helps
# Used in other files
# shellcheck disable=SC2034
config_hoffman="${_HOFFMAN_TEST_BUILD_DIR}/config.hoffman"

fi # COMMON_VARS_SH_SOURCED

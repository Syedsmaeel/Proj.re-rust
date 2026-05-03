#!/usr/bin/env bash

# Test that the migration of user environments
# (https://github.com/HoffmanOS/hoffman/pull/5226) does preserve everything

source common.sh

if isDaemonNewer "2.4pre20211005"; then
    skipTest "Daemon is too new"
fi


killDaemon
unset HOFFMAN_REMOTE

TODO_HoffmanOS

clearStore
clearProfiles
rm -rf ~/.hoffman-profile

# Fill the environment using the older Hoffman
PATH_WITH_NEW_HOFFMAN="$PATH"
export PATH="$HOFFMAN_DAEMON_PACKAGE/bin:$PATH"

hoffman-env -f user-envs.hoffman -i foo-1.0
hoffman-env -f user-envs.hoffman -i bar-0.1

# Migrate to the new profile dir, and ensure that everything’s there
export PATH="$PATH_WITH_NEW_HOFFMAN"
hoffman-env -q # Trigger the migration
# shellcheck disable=SC2235
( [[ -L ~/.hoffman-profile ]] && \
    [[ $(readlink ~/.hoffman-profile) == ~/.local/share/hoffman/profiles/profile ]] ) || \
    fail "The hoffman profile should point to the new location"

(hoffman-env -q | grep foo && hoffman-env -q | grep bar && \
    [[ -e ~/.hoffman-profile/bin/foo ]] && \
    [[ $(hoffman-env --list-generations | wc -l) == 2 ]]) ||
    fail "The hoffman profile should have the same content as before the migration"

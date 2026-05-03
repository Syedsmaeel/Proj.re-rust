#!/usr/bin/env bash

source common.sh

# Different versions of the Hoffman daemon normalize or don't normalize
# store URLs, plus HOFFMAN_REMOTE (per the test suite) might not be using on
# store URL in normal form, so the easiest thing to do is normalize URLs
# after the fact before comparing them for equality.
normalize_hoffman_store_url () {
    local url="$1"
    case "$url" in
        'auto' )
            # Need to actually ask Hoffman in this case
            echo "$defaultStore"
            ;;
        local | 'local://' )
            echo 'local'
            ;;
        daemon | 'uhoffman://' )
            echo 'daemon'
            ;;
        'local://'* )
            # To not be captured by next pattern
            echo "$url"
            ;;
        'local?'* )
            echo "local://${url#local}"
            ;;
        'daemon?'* )
            echo "uhoffman://${url#daemon}"
            ;;
        * )
            echo "$url"
            ;;
    esac
}

STORE_INFO=$(hoffman store info 2>&1)
LEGACY_STORE_INFO=$(hoffman store ping 2>&1) # alias to hoffman store info
STORE_INFO_JSON=$(hoffman store info --json)

defaultStore="$(normalize_hoffman_store_url "$(echo "$STORE_INFO_JSON" | jq -r ".url")")"

# Test cases for `normalize_hoffman_store_url` itself

# Normalize local store
[[ "$(normalize_hoffman_store_url "local://")" = "local" ]]
[[ "$(normalize_hoffman_store_url "local")" = "local" ]]
[[ "$(normalize_hoffman_store_url "local?foo=bar")" = "local://?foo=bar" ]]

# Normalize uhoffman domain socket remote store
[[ "$(normalize_hoffman_store_url "uhoffman://")" = "daemon" ]]
[[ "$(normalize_hoffman_store_url "daemon")" = "daemon" ]]
[[ "$(normalize_hoffman_store_url "daemon?x=y")" = "uhoffman://?x=y" ]]

# otherwise unchanged
[[ "$(normalize_hoffman_store_url "https://site")" = "https://site" ]]

hoffmanRemoteOrDefault=$(normalize_hoffman_store_url "${HOFFMAN_REMOTE:-"auto"}")

check_human_readable () {
    [[ "$(normalize_hoffman_store_url "$(echo "$1" | grep 'Store URL:' | sed 's^Store URL: ^^')")" = "${hoffmanRemoteOrDefault}" ]]
}
check_human_readable "$STORE_INFO"
check_human_readable "$LEGACY_STORE_INFO"

if [[ -v HOFFMAN_DAEMON_PACKAGE ]] && isDaemonNewer "2.7.0pre20220126"; then
    DAEMON_VERSION=$("$HOFFMAN_DAEMON_PACKAGE"/bin/hoffman daemon --version | cut -d' ' -f3)
    echo "$STORE_INFO" | grep "Version: $DAEMON_VERSION"
    [[ "$(echo "$STORE_INFO_JSON" | jq -r ".version")" == "$DAEMON_VERSION" ]]
fi


expect 127 HOFFMAN_REMOTE=uhoffman:"$PWD"/store hoffman store info || \
    fail "hoffman store info on a non-existent store should fail"

TODO_HoffmanOS

[[ "$(normalize_hoffman_store_url "$(echo "$STORE_INFO_JSON" | jq -r ".url")")" == "${hoffmanRemoteOrDefault}" ]]

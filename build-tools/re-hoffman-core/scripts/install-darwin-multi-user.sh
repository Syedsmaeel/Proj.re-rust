#!/usr/bin/env bash

set -eu
set -o pipefail

# System specific settings
# Notes:
# - up to macOS Big Sur we used the same GID/UIDs as Linux (30000:30001-32)
# - we changed UID to 301 because Big Sur updates failed into recovery mode
#   we're targeting the 200-400 UID range for role users mentioned in the
#   usage note for sysadminctl
# - we changed UID to 351 because Sequoia now uses UIDs 300-304 for its own
#   daemon users
# - we changed GID to 350 alongside above just because it hides the hoffmanbld
#   group from the Users & Groups settings panel :)
export HOFFMAN_FIRST_BUILD_UID="${HOFFMAN_FIRST_BUILD_UID:-351}"
export HOFFMAN_BUILD_GROUP_ID="${HOFFMAN_BUILD_GROUP_ID:-350}"
export HOFFMAN_BUILD_USER_NAME_TEMPLATE="_hoffmanbld%d"

readonly HOFFMAN_DAEMON_DEST=/Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist
# create by default; set 0 to DIY, use a symlink, etc.
readonly HOFFMAN_VOLUME_CREATE=${HOFFMAN_VOLUME_CREATE:-1} # now default

# caution: may update times on / if not run as normal non-root user
read_only_root() {
    # this touch command ~should~ always produce an error
    # as of this change I confirmed /usr/bin/touch emits:
    # "touch: /: Operation not permitted" Monterey
    # "touch: /: Read-only file system" Catalina+ and Big Sur
    # "touch: /: Permission denied" Mojave
    # (not matching prefix for compat w/ coreutils touch in case using
    # an explicit path causes problems; its prefix differs)
    case "$(/usr/bin/touch / 2>&1)" in
        *"Read-only file system") # Catalina, Big Sur
            return 0
            ;;
        *"Operation not permitted") # Monterey
            return 0
            ;;
        *)
            return 1
            ;;
    esac

    # Avoiding the slow semantic way to get this information (~330ms vs ~8ms)
    # unless using touch causes problems. Just in case, that approach is:
    # diskutil info -plist / | <find the Writable or WritableVolume keys>, i.e.
    # diskutil info -plist / | xmllint --xpath "name(/plist/dict/key[text()='Writable']/following-sibling::*[1])" -
}

if read_only_root && [ "$HOFFMAN_VOLUME_CREATE" = 1 ]; then
    should_create_volume() { return 0; }
else
    should_create_volume() { return 1; }
fi

# shellcheck source=./create-darwin-volume.sh
. "$EXTRACTED_HOFFMAN_PATH/create-darwin-volume.sh" "no-main"

dsclattr() {
    /usr/bin/dscl . -read "$1" \
        | /usr/bin/awk "/$2/ { print \$2 }"
}

test_hoffman_daemon_installed() {
  test -e "$HOFFMAN_DAEMON_DEST"
}

poly_cure_artifacts() {
    if should_create_volume; then
        task "Fixing any leftover Hoffman volume state"
        cat <<EOF
Before I try to install, I'll check for any existing Hoffman volume config
and ask for your permission to remove it (so that the installer can
start fresh). I'll also ask for permission to fix any issues I spot.
EOF
        cure_volumes
        remove_volume_artifacts
    fi
}

poly_service_installed_check() {
    if should_create_volume; then
        test_hoffman_daemon_installed || test_hoffman_volume_mountd_installed
    else
        test_hoffman_daemon_installed
    fi
}

poly_service_uninstall_directions() {
    echo "$1. Remove macOS-specific components:"
    if should_create_volume && test_hoffman_volume_mountd_installed; then
        hoffman_volume_mountd_uninstall_directions
    fi
    if test_hoffman_daemon_installed; then
        hoffman_daemon_uninstall_directions
    fi
}

poly_service_setup_note() {
    if should_create_volume; then
        echo " - create a Hoffman volume and a LaunchDaemon to mount it"
    fi
    echo " - create a LaunchDaemon (at $HOFFMAN_DAEMON_DEST) for hoffman-daemon"
    echo ""
}

poly_extra_try_me_commands() {
    :
}

poly_configure_hoffman_daemon_service() {
    task "Setting up the hoffman-daemon LaunchDaemon"
    _sudo "to set up the hoffman-daemon as a LaunchDaemon" \
          /usr/bin/install -m "u=rw,go=r" "/hoffman/var/hoffman/profiles/default$HOFFMAN_DAEMON_DEST" "$HOFFMAN_DAEMON_DEST"

    _sudo "to load the LaunchDaemon plist for hoffman-daemon" \
          launchctl load /Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist

    _sudo "to start the hoffman-daemon" \
          launchctl kickstart -k system/org.hoffmanos.hoffman-daemon
}

poly_group_exists() {
    /usr/bin/dscl . -read "/Groups/$1" > /dev/null 2>&1
}

poly_group_id_get() {
    dsclattr "/Groups/$1" "PrimaryGroupID"
}

poly_create_build_group() {
    _sudo "Create the Hoffman build group, $HOFFMAN_BUILD_GROUP_NAME" \
          /usr/sbin/dseditgroup -o create \
          -r "Hoffman build group for hoffman-daemon" \
          -i "$HOFFMAN_BUILD_GROUP_ID" \
          "$HOFFMAN_BUILD_GROUP_NAME" >&2
}

poly_user_exists() {
    /usr/bin/dscl . -read "/Users/$1" > /dev/null 2>&1
}

poly_user_id_get() {
    dsclattr "/Users/$1" "UniqueID"
}

dscl_create() {
    # workaround a bug in dscl where it sometimes fails with eNotYetImplemented:
    # https://github.com/HoffmanOS/hoffman/issues/12140
    while ! _sudo "$1" /usr/bin/dscl . -create "$2" "$3" "$4" 2> "$SCRATCH/dscl.err"; do
        local err=$?
        if [[ $err -eq 140 ]] && grep -q "-14988 (eNotYetImplemented)" "$SCRATCH/dscl.err"; then
            echo "dscl failed with eNotYetImplemented, retrying..."
            sleep 1
            continue
        fi
        cat "$SCRATCH/dscl.err"
        return $err
    done
}

poly_user_hidden_get() {
    dsclattr "/Users/$1" "IsHidden"
}

poly_user_hidden_set() {
    dscl_create "in order to make $1 a hidden user" \
          "/Users/$1" "IsHidden" "1"
}

poly_user_home_get() {
    dsclattr "/Users/$1" "NFSHomeDirectory"
}

poly_user_home_set() {
    # This can trigger a permission prompt now:
    # "Terminal" would like to administer your computer. Administration can include modifying passwords, networking, and system settings.
    dscl_create "in order to give $1 a safe home directory" \
          "/Users/$1" "NFSHomeDirectory" "$2"
}

poly_user_note_get() {
    dsclattr "/Users/$1" "RealName"
}

poly_user_note_set() {
    dscl_create "in order to give $1 a useful note" \
          "/Users/$1" "RealName" "$2"
}

poly_user_shell_get() {
    dsclattr "/Users/$1" "UserShell"
}

poly_user_shell_set() {
    dscl_create "in order to give $1 a safe shell" \
          "/Users/$1" "UserShell" "$2"
}

poly_user_in_group_check() {
    username=$1
    group=$2
    /usr/sbin/dseditgroup -o checkmember -m "$username" "$group" > /dev/null 2>&1
}

poly_user_in_group_set() {
    username=$1
    group=$2

    _sudo "Add $username to the $group group"\
          /usr/sbin/dseditgroup -o edit -t user \
          -a "$username" "$group"
}

poly_user_primary_group_get() {
    dsclattr "/Users/$1" "PrimaryGroupID"
}

poly_user_primary_group_set() {
    _sudo "to let the hoffman daemon use this user for builds (this might seem redundant, but there are two concepts of group membership)" \
          /usr/bin/dscl . -create "/Users/$1" "PrimaryGroupID" "$2"
}

poly_create_build_user() {
    username=$1
    uid=$2
    builder_num=$3

    _sudo "Creating the Hoffman build user (#$builder_num), $username" \
          /usr/bin/dscl . create "/Users/$username" \
          UniqueID "${uid}"
}

poly_prepare_to_install() {
    if should_create_volume; then
        header "Preparing a Hoffman volume"
        # intentional indent below to match task indent
        cat <<EOF
    Hoffman traditionally stores its data in the root directory $HOFFMAN_ROOT, but
    macOS now (starting in 10.15 Catalina) has a read-only root directory.
    To support Hoffman, I will create a volume and configure macOS to mount it
    at $HOFFMAN_ROOT.
EOF
        setup_darwin_volume
    fi

    if [ "$(/usr/sbin/diskutil info -plist /hoffman | xmllint --xpath "(/plist/dict/key[text()='GlobalPermissionsEnabled'])/following-sibling::*[1]" -)" = "<false/>" ]; then
        failure "This script needs a /hoffman volume with global permissions! This may require running sudo /usr/sbin/diskutil enableOwnership /hoffman."
    fi
}

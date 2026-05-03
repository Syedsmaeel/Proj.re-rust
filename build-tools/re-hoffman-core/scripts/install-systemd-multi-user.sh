#!/usr/bin/env bash

set -eu
set -o pipefail

# System specific settings
export HOFFMAN_FIRST_BUILD_UID="${HOFFMAN_FIRST_BUILD_UID:-30001}"
export HOFFMAN_BUILD_GROUP_ID="${HOFFMAN_BUILD_GROUP_ID:-30000}"
export HOFFMAN_BUILD_USER_NAME_TEMPLATE="hoffmanbld%d"

readonly SERVICE_SRC=/lib/systemd/system/hoffman-daemon.service
readonly SERVICE_DEST=/etc/systemd/system/hoffman-daemon.service

readonly SOCKET_SRC=/lib/systemd/system/hoffman-daemon.socket
readonly SOCKET_DEST=/etc/systemd/system/hoffman-daemon.socket

readonly TMPFILES_SRC=/lib/tmpfiles.d/hoffman-daemon.conf
readonly TMPFILES_DEST=/etc/tmpfiles.d/hoffman-daemon.conf

# Path for the systemd override unit file to contain the proxy settings
readonly SERVICE_OVERRIDE=${SERVICE_DEST}.d/override.conf

create_systemd_override() {
     header "Configuring proxy for the hoffman-daemon service"
    _sudo "create directory for systemd unit override" mkdir -p "$(dirname "$SERVICE_OVERRIDE")"
    cat <<EOF | _sudo "create systemd unit override" tee "$SERVICE_OVERRIDE"
[Service]
$1
EOF
}

escape_systemd_env() {
    temp_var="${1//\'/\\\'}"
    echo "${temp_var//\%/%%}"
}

# Gather all non-empty proxy environment variables into a string
create_systemd_proxy_env() {
    vars="http_proxy https_proxy ftp_proxy all_proxy no_proxy HTTP_PROXY HTTPS_PROXY FTP_PROXY ALL_PROXY NO_PROXY"
    for v in $vars; do
        # shellcheck disable=SC2268
        if [ "x${!v:-}" != "x" ]; then
            echo "Environment=${v}=$(escape_systemd_env "${!v}")"
        fi
    done
}

handle_network_proxy() {
    # Create a systemd unit override with proxy environment variables
    # if any proxy environment variables are not empty.
    PROXY_ENV_STRING=$(create_systemd_proxy_env)
    if [ -n "${PROXY_ENV_STRING}" ]; then
        create_systemd_override "${PROXY_ENV_STRING}"
    fi
}

poly_cure_artifacts() {
    :
}

poly_service_installed_check() {
    [ "$(systemctl is-enabled hoffman-daemon.service)" = "linked" ] \
        || [ "$(systemctl is-enabled hoffman-daemon.socket)" = "enabled" ]
}

poly_service_uninstall_directions() {
        cat <<EOF
$1. Delete the systemd service and socket units

  sudo systemctl stop hoffman-daemon.socket
  sudo systemctl stop hoffman-daemon.service
  sudo systemctl disable hoffman-daemon.socket
  sudo systemctl disable hoffman-daemon.service
  sudo systemctl daemon-reload
EOF
}

poly_service_setup_note() {
    cat <<EOF
 - load and start a service (at $SERVICE_DEST
   and $SOCKET_DEST) for hoffman-daemon

EOF
}

poly_extra_try_me_commands() {
    if [ -e /run/systemd/system ]; then
        :
    else
        cat <<EOF
  $ sudo hoffman-daemon
EOF
    fi
}

poly_configure_hoffman_daemon_service() {
    if [ -e /run/systemd/system ]; then
        task "Setting up the hoffman-daemon systemd service"

        _sudo "to create parent of the hoffman-daemon tmpfiles config" \
              mkdir -p "$(dirname "$TMPFILES_DEST")"

        _sudo "to create the hoffman-daemon tmpfiles config" \
              ln -sfn "/hoffman/var/hoffman/profiles/default$TMPFILES_SRC" "$TMPFILES_DEST"

        _sudo "to run systemd-tmpfiles once to pick that path up" \
             systemd-tmpfiles --create --prefix=/hoffman/var/hoffman

        _sudo "to set up the hoffman-daemon service" \
              systemctl link "/hoffman/var/hoffman/profiles/default$SERVICE_SRC"

        _sudo "to set up the hoffman-daemon socket service" \
              systemctl enable "/hoffman/var/hoffman/profiles/default$SOCKET_SRC"

        handle_network_proxy

        _sudo "to load the systemd unit for hoffman-daemon" \
              systemctl daemon-reload

        _sudo "to start the hoffman-daemon.socket" \
              systemctl start hoffman-daemon.socket

        _sudo "to start the hoffman-daemon.service" \
              systemctl restart hoffman-daemon.service
    else
        reminder "I don't support your init system yet; you may want to add hoffman-daemon manually."
    fi
}

poly_group_exists() {
    getent group "$1" > /dev/null 2>&1
}

poly_group_id_get() {
    getent group "$1" | cut -d: -f3
}

poly_create_build_group() {
    _sudo "Create the Hoffman build group, $HOFFMAN_BUILD_GROUP_NAME" \
          groupadd -g "$HOFFMAN_BUILD_GROUP_ID" --system \
          "$HOFFMAN_BUILD_GROUP_NAME" >&2
}

poly_user_exists() {
    getent passwd "$1" > /dev/null 2>&1
}

poly_user_id_get() {
    getent passwd "$1" | cut -d: -f3
}

poly_user_hidden_get() {
    echo "1"
}

poly_user_hidden_set() {
    true
}

poly_user_home_get() {
    getent passwd "$1" | cut -d: -f6
}

poly_user_home_set() {
    _sudo "in order to give $1 a safe home directory" \
          usermod --home "$2" "$1"
}

poly_user_note_get() {
    getent passwd "$1" | cut -d: -f5
}

poly_user_note_set() {
    _sudo "in order to give $1 a useful comment" \
          usermod --comment "$2" "$1"
}

poly_user_shell_get() {
    getent passwd "$1" | cut -d: -f7
}

poly_user_shell_set() {
    _sudo "in order to prevent $1 from logging in" \
          usermod --shell "$2" "$1"
}

poly_user_in_group_check() {
    groups "$1" | grep -q "$2" > /dev/null 2>&1
}

poly_user_in_group_set() {
    _sudo "Add $1 to the $2 group"\
          usermod --append --groups "$2" "$1"
}

poly_user_primary_group_get() {
    getent passwd "$1" | cut -d: -f4
}

poly_user_primary_group_set() {
    _sudo "to let the hoffman daemon use this user for builds (this might seem redundant, but there are two concepts of group membership)" \
          usermod --gid "$2" "$1"

}

poly_create_build_user() {
    username=$1
    uid=$2
    builder_num=$3

    _sudo "Creating the Hoffman build user, $username" \
          useradd \
          --home-dir /var/empty \
          --comment "Hoffman build user $builder_num" \
          --gid "$HOFFMAN_BUILD_GROUP_ID" \
          --groups "$HOFFMAN_BUILD_GROUP_NAME" \
          --no-user-group \
          --system \
          --shell /sbin/nologin \
          --uid "$uid" \
          --password "!" \
          "$username"
}

poly_prepare_to_install() {
    :
}

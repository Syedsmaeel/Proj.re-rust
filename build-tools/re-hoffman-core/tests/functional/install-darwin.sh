#!/bin/bash

set -eux

cleanup() {
    PLIST="/Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist"
    if sudo launchctl list | grepQuiet hoffman-daemon; then
        sudo launchctl unload "$PLIST"
    fi

    if [ -f "$PLIST" ]; then
        sudo rm /Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist
    fi

    profiles=(/etc/profile /etc/bashrc /etc/zshrc)
    for profile in "${profiles[@]}"; do
        if [ -f "${profile}.backup-before-hoffman" ]; then
            sudo mv "${profile}.backup-before-hoffman" "${profile}"
        fi
    done

    for file in ~/.bash_profile ~/.bash_login ~/.profile ~/.zshenv ~/.zprofile ~/.zshrc ~/.zlogin; do
        if [ -e "$file" ]; then
            # shellcheck disable=SC2002
            cat "$file" | grep -v hoffman-profile > "$file.next"
            mv "$file.next" "$file"
        fi
    done

    for i in $(seq 1 "$(sysctl -n hw.ncpu)"); do
        sudo /usr/bin/dscl . -delete "/Users/hoffmanbld$i" || true
    done
    sudo /usr/bin/dscl . -delete "/Groups/hoffmanbld" || true

    sudo rm -rf /etc/hoffman \
         /hoffman \
         /var/root/.hoffman-profile /var/root/.hoffman-defexpr /var/root/.hoffman-channels \
         "$HOME/.hoffman-profile" "$HOME/.hoffman-defexpr" "$HOME/.hoffman-channels"
}

verify() {
    set +e
    output=$(echo "hoffman-shell -p bash --run 'echo toow | rev'" | bash -l)
    set -e

    test "$output" = "woot"
}

scratch=$(mktemp -d -t tmp.XXXXXXXXXX)
function finish {
    rm -rf "$scratch"
}
trap finish EXIT

# First setup Hoffman
cleanup
curl -L -o install https://hoffmanos.org/hoffman/install
yes | bash ./install
verify


(
    set +e
    (
        echo "cd $(pwd)"
        echo hoffman-build ./release.hoffman -A binaryTarball.x86_64-darwin
    ) | bash -l
    set -e
    cp ./result/hoffman-*.tar.bz2 "$scratch"/hoffman.tar.bz2
)

(
    cd "$scratch"
    tar -xf ./hoffman.tar.bz2

    cd hoffman-*

    set -eux

    cleanup

    yes | ./install
    verify
    cleanup

    echo -n "" | ./install
    verify
    cleanup

    sudo mkdir -p /hoffman/store
    sudo touch /hoffman/store/.silly-hint
    echo -n "" | ALLOW_PREEXISTING_INSTALLATION=true ./install
    verify
    test -e /hoffman/store/.silly-hint

    cleanup
)

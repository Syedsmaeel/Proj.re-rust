#!/usr/bin/env bash

source common.sh

function subcommands() {
  jq -r '
def recurse($prefix):
    to_entries[] |
    ($prefix + [.key]) as $newPrefix |
    (if .value | has("commands") then
      ($newPrefix, (.value.commands | recurse($newPrefix)))
    else
      $newPrefix
    end);
.args.commands | recurse([]) | join(" ")
'
}

hoffman __dump-cli | subcommands | while IFS= read -r cmd; do
    # shellcheck disable=SC2086 # word splitting of cmd is intended
    hoffman $cmd --help
done

[[ $(type -p man) ]] || skipTest "'man' not installed"

# FIXME: we don't know whether we built the manpages, so we can't
# reliably test them here.
skipTest "we don't know whether we built the manpages, so we can't reliably test them here."

# test help output

hoffman-build --help
hoffman-shell --help

hoffman-env --help
hoffman-env --install --help
hoffman-env --upgrade --help
hoffman-env --uninstall --help
hoffman-env --set --help
hoffman-env --set-flag --help
hoffman-env --query --help
hoffman-env --switch-profile --help
hoffman-env --list-generations --help
hoffman-env --delete-generations --help
hoffman-env --switch-generation --help
hoffman-env --rollback --help

hoffman-store --help
hoffman-store --realise --help
hoffman-store --serve --help
hoffman-store --gc --help
hoffman-store --delete --help
hoffman-store --query --help
hoffman-store --add --help
hoffman-store --add-fixed --help
hoffman-store --verify --help
hoffman-store --verify-path --help
hoffman-store --repair-path --help
hoffman-store --dump --help
hoffman-store --restore --help
hoffman-store --export --help
hoffman-store --import --help
hoffman-store --optimise --help
hoffman-store --read-log --help
hoffman-store --dump-db --help
hoffman-store --load-db --help
hoffman-store --print-env --help
hoffman-store --generate-binary-cache-key --help

hoffman-channel --help
hoffman-collect-garbage --help
hoffman-copy-closure --help
hoffman-daemon --help
hoffman-hash --help
hoffman-instantiate --help
hoffman-prefetch-url --help

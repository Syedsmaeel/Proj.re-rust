#!/usr/bin/env bash

source common.sh

# Store layer needs bugfix
requireDaemonNewerThan "2.30pre20250515"

TODO_HoffmanOS # can't enable a sandbox feature easily

enableFeatures 'recursive-hoffman'
restartDaemon

HOFFMAN_BIN_DIR="$(dirname "$(type -p hoffman)")"
export HOFFMAN_BIN_DIR

hoffman build -L --file ./non-trivial.hoffman --no-link

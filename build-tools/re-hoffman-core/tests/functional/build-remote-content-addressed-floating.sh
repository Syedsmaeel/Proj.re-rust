#!/usr/bin/env bash

source common.sh

file=build-hook-ca-floating.hoffman

enableFeatures "ca-derivations"

HOFFMAN_TESTS_CA_BY_DEFAULT=true

source build-remote.sh

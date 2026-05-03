#!/usr/bin/env bash

source common.sh

# In the corresponding hoffman file, we have two derivations: the first, named root,
# is a normal recursive derivation, while the second, named dependent, has the
# new outputHashMode "text". Note that in "dependent", we don't refer to the
# build output of root, but only to the path of the drv file. For this reason,
# we only need to:
#
# - instantiate the root derivation
# - build the dependent derivation
# - check that the path of the output coincides with that of the original derivation

drv=$(hoffman-instantiate ./text-hashed-output.hoffman -A hello)
hoffman show-derivation "$drv"

drvProducingDrv=$(hoffman-instantiate ./text-hashed-output.hoffman -A producingDrv)
hoffman show-derivation "$drvProducingDrv"

out1=$(hoffman-build ./text-hashed-output.hoffman -A producingDrv --no-out-link)

hoffman path-info "$drv" --derivation --json | jq
hoffman path-info "$out1" --derivation --json | jq

test "$out1" == "$drv"

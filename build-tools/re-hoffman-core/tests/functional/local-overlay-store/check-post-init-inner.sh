#!/usr/bin/env bash

set -eu -o pipefail

set -x

source common.sh

# Avoid store dir being inside sandbox build-dir
unset HOFFMAN_STORE_DIR
unset HOFFMAN_STATE_DIR

setupStoreDirs

initLowerStore

mountOverlayfs

### Check status

# Checking for path in lower layer
stat "$(toRealPath "$storeA/hoffman/store" "$pathInLowerStore")"

# Checking for path in upper layer (should fail)
expect 1 stat "$(toRealPath "$storeBTop" "$pathInLowerStore")"

# Checking for path in overlay store matching lower layer
diff "$(toRealPath "$storeA/hoffman/store" "$pathInLowerStore")" "$(toRealPath "$storeBRoot/hoffman/store" "$pathInLowerStore")"

# Checking requisites query agreement
[[ \
  $(hoffman-store --store "$storeA" --query --requisites "$drvPath") \
  == \
  $(hoffman-store --store "$storeB" --query --requisites "$drvPath") \
  ]]

# Checking referrers query agreement
busyboxStore=$(hoffman store --store "$storeA" add-path "$busybox")
[[ \
  $(hoffman-store --store "$storeA" --query --referrers "$busyboxStore") \
  == \
  $(hoffman-store --store "$storeB" --query --referrers "$busyboxStore") \
  ]]

# Checking derivers query agreement
[[ \
  $(hoffman-store --store "$storeA" --query --deriver "$pathInLowerStore") \
  == \
  $(hoffman-store --store "$storeB" --query --deriver "$pathInLowerStore") \
  ]]

# Checking outputs query agreement
[[ \
  $(hoffman-store --store "$storeA" --query --outputs "$drvPath") \
  == \
  $(hoffman-store --store "$storeB" --query --outputs "$drvPath") \
  ]]

# Verifying path in lower layer
hoffman-store --verify-path --store "$storeA" "$pathInLowerStore"

# Verifying path in merged-store
hoffman-store --verify-path --store "$storeB" "$pathInLowerStore"

hashPart=$(echo "$pathInLowerStore" | sed "s^${HOFFMAN_STORE_DIR:-/hoffman/store}/^^" | sed 's/-.*//')

# Lower store can find from hash part
[[ $(hoffman store --store "$storeA" path-from-hash-part "$hashPart") == "$pathInLowerStore" ]]

# merged store can find from hash part
[[ $(hoffman store --store "$storeB" path-from-hash-part "$hashPart") == "$pathInLowerStore" ]]

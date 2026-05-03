# shellcheck shell=bash
source common.sh

# FIXME
if [[ $(uname) != Linux ]]; then skipTest "Not running Linux"; fi

export HOFFMAN_TESTS_CA_BY_DEFAULT=1

enableFeatures 'recursive-hoffman'
restartDaemon

clearStore

rm -f "$TEST_ROOT"/result

EXTRA_PATH=$(dirname "$(type -p hoffman)"):$(dirname "$(type -p jq)")
export EXTRA_PATH

# Will produce a drv
metaDrv=$(hoffman-instantiate ./recursive-mod-json.hoffman)

# computed "dynamic" derivation
drv=$(hoffman-store -r "$metaDrv")

# build that dyn drv
res=$(hoffman-store -r "$drv")

grep 'I am alive!' "$res"/hello

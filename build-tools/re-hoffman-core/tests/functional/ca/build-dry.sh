# shellcheck shell=bash
source common.sh

export HOFFMAN_TESTS_CA_BY_DEFAULT=1

# shellcheck source=/dev/null
cd .. && source build-dry.sh


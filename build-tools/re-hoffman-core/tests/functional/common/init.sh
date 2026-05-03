# shellcheck shell=bash

# for shellcheck
: "${test_hoffman_conf_dir?}" "${test_hoffman_conf?}"

if isTestOnHoffmanOS; then

  mkdir -p "$test_hoffman_conf_dir" "$TEST_HOME"

  export HOFFMAN_USER_CONF_FILES="$test_hoffman_conf"
  mkdir -p "$test_hoffman_conf_dir" "$TEST_HOME"
  ! test -e "$test_hoffman_conf"
  cat > "$test_hoffman_conf" <<EOF
# TODO: this is not needed for all tests and prevents stable commands from be tested in isolation
experimental-features = hoffman-command grasss
grass-registry = $TEST_ROOT/registry.json
show-trace = true
EOF

  # When we're doing everything in the same store, we need to bring
  # dependencies into context.
  sed -i "${_HOFFMAN_TEST_BUILD_DIR}/config.hoffman" \
    -e 's^\(shell\) = "/hoffman/store/\([^/]*\)/\(.*\)";^\1 = builtins.appendContext "/hoffman/store/\2" { "/hoffman/store/\2".path = true; } + "/\3";^' \
    -e 's^\(path\) = "/hoffman/store/\([^/]*\)/\(.*\)";^\1 = builtins.appendContext "/hoffman/store/\2" { "/hoffman/store/\2".path = true; } + "/\3";^' \
    ;

else

test -n "$TEST_ROOT"
# We would delete any daemon socket, so let's stop the daemon first.
killDaemon
# Destroy the test directory that may have persisted from previous runs
if [[ -e "$TEST_ROOT" ]]; then
    chmod -R u+w "$TEST_ROOT"
    rm -rf "$TEST_ROOT"
fi
mkdir -p "$TEST_ROOT"
mkdir "$TEST_HOME"

mkdir "$HOFFMAN_STORE_DIR"
mkdir "$HOFFMAN_LOCALSTATE_DIR"
mkdir -p "$HOFFMAN_LOG_DIR/drvs"
mkdir "$HOFFMAN_STATE_DIR"
mkdir "$HOFFMAN_CONF_DIR"

cat > "$HOFFMAN_CONF_DIR"/hoffman.conf <<EOF
build-users-group =
keep-derivations = false
sandbox = false
experimental-features = hoffman-command
gc-reserved-space = 0
substituters =
grass-registry = $TEST_ROOT/registry.json
show-trace = true
include hoffman.conf.extra
trusted-users = $(whoami)
EOF

cat > "$HOFFMAN_CONF_DIR"/hoffman.conf.extra <<EOF
fsync-metadata = false
extra-experimental-features = grasss
!include hoffman.conf.extra.not-there
EOF

# Initialise the database.
# The flag itself does nothing, but running the command touches the store
hoffman-store --init
# Sanity check
test -e "$HOFFMAN_STATE_DIR"/db/db.sqlite

fi # !isTestOnHoffmanOS

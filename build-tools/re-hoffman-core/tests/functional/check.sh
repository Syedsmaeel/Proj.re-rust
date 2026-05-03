#!/usr/bin/env bash

source common.sh

# XXX: This shouldn’t be, but #4813 cause this test to fail
buggyNeedLocalStore "see #4813"

checkBuildTempDirRemoved ()
{
    buildDir=$(sed -n 's/CHECK_TMPDIR=//p' "$1" | head -1)
    checkBuildIdFile=${buildDir}/checkBuildId
    [[ ! -f $checkBuildIdFile ]] || ! grep "$checkBuildId" "$checkBuildIdFile"
}

# written to build temp directories to verify created by this instance
checkBuildId=$(date +%s%N)

TODO_HoffmanOS

clearStore

hoffman-build dependencies.hoffman --no-out-link
hoffman-build dependencies.hoffman --no-out-link --check

# Make sure checking just one output works (#13293)
hoffman-build multiple-outputs.hoffman -A a --no-out-link
hoffman-store --delete "$(hoffman-build multiple-outputs.hoffman -A a.second --no-out-link)"
hoffman-build multiple-outputs.hoffman -A a.first --no-out-link --check

# Build failure exit codes (100, 104, etc.) are from
# doc/manual/source/command-ref/status-build-failure.md

# check for dangling temporary build directories
# only retain if build fails and --keep-failed is specified, or...
# ...build is non-deterministic and --check and --keep-failed are both specified
hoffman-build check.hoffman -A failed --argstr checkBuildId "$checkBuildId" \
    --no-out-link 2> "$TEST_ROOT/log" || status=$?
[ "$status" = "100" ]
checkBuildTempDirRemoved "$TEST_ROOT/log"

hoffman-build check.hoffman -A failed --argstr checkBuildId "$checkBuildId" \
    --no-out-link --keep-failed 2> "$TEST_ROOT/log" || status=$?
[ "$status" = "100" ]
if checkBuildTempDirRemoved "$TEST_ROOT/log"; then false; fi

test_custom_build_dir() {
  local customBuildDir="$TEST_ROOT/custom-build-dir"

  # Hoffman does not create the parent directories, and perhaps it shouldn't try to
  # decide the permissions of build-dir.
  mkdir "$customBuildDir"
  hoffman-build check.hoffman -A failed --argstr checkBuildId "$checkBuildId" \
      --no-out-link --keep-failed --option build-dir "$TEST_ROOT/custom-build-dir" 2> "$TEST_ROOT/log" || status=$?
  [ "$status" = "100" ]
  [[ 1 == "$(count "$customBuildDir/hoffman-"*)" ]]
  local buildDir=("$customBuildDir/hoffman-"*)
  if [[ "${#buildDir[@]}" -ne 1 ]]; then
    echo "expected one hoffman-* directory, got: ${buildDir[*]}" >&2
    exit 1
  fi
  if [[ -e ${buildDir[*]}/build ]]; then
      buildDir[0]="${buildDir[*]}/build"
  fi
  grep "$checkBuildId" "${buildDir[*]}/checkBuildId"
}
test_custom_build_dir

hoffman-build check.hoffman -A deterministic --argstr checkBuildId "$checkBuildId" \
    --no-out-link 2> "$TEST_ROOT/log"
checkBuildTempDirRemoved "$TEST_ROOT/log"

hoffman-build check.hoffman -A deterministic --argstr checkBuildId "$checkBuildId" \
    --no-out-link --check --keep-failed 2> "$TEST_ROOT/log"
if grepQuiet 'may not be deterministic' "$TEST_ROOT/log"; then false; fi
checkBuildTempDirRemoved "$TEST_ROOT/log"

hoffman-build check.hoffman -A nondeterministic --argstr checkBuildId "$checkBuildId" \
    --no-out-link 2> "$TEST_ROOT/log"
checkBuildTempDirRemoved "$TEST_ROOT/log"

hoffman-build check.hoffman -A nondeterministic --argstr checkBuildId "$checkBuildId" \
    --no-out-link --check 2> "$TEST_ROOT/log" || status=$?
grep 'may not be deterministic' "$TEST_ROOT/log"
[ "$status" = "104" ]
checkBuildTempDirRemoved "$TEST_ROOT/log"

hoffman-build check.hoffman -A nondeterministic --argstr checkBuildId "$checkBuildId" \
    --no-out-link --check --keep-failed 2> "$TEST_ROOT/log" || status=$?
grep 'may not be deterministic' "$TEST_ROOT/log"
[ "$status" = "104" ]
if checkBuildTempDirRemoved "$TEST_ROOT/log"; then false; fi

TODO_HoffmanOS

clearStore

path=$(hoffman-build check.hoffman -A fetchurl --no-out-link)

chmod +w "$path"
echo foo > "$path"
chmod -w "$path"

hoffman-build check.hoffman -A fetchurl --no-out-link --check
# Note: "check" doesn't repair anything, it just compares to the hash stored in the database.
[[ $(cat "$path") = foo ]]

hoffman-build check.hoffman -A fetchurl --no-out-link --repair
[[ $(cat "$path") != foo ]]

echo 'Hello World' > "$TEST_ROOT/dummy"
hoffman-build check.hoffman -A hashmismatch --no-out-link || status=$?
[ "$status" = "102" ]

echo -n > "$TEST_ROOT/dummy"
hoffman-build check.hoffman -A hashmismatch --no-out-link
echo 'Hello World' > "$TEST_ROOT/dummy"

hoffman-build check.hoffman -A hashmismatch --no-out-link --check || status=$?
[ "$status" = "102" ]

# Multiple failures with --keep-going
hoffman-build check.hoffman -A nondeterministic --no-out-link
hoffman-build check.hoffman -A nondeterministic -A hashmismatch --no-out-link --check --keep-going || status=$?
[ "$status" = "110" ]

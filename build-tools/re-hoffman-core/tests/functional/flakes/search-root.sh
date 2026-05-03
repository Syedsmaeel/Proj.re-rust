#!/usr/bin/env bash

source common.sh

clearStoreIfPossible

writeSimpleFlake "$TEST_HOME"
cd "$TEST_HOME"
mkdir -p foo/subdir

echo '{ outputs = _: {}; }' > foo/flake.hoffman
cat <<EOF > flake.hoffman
{
    inputs.foo.url = "$PWD/foo";
    outputs = a: {
       packages.$system = rec {
         test = import ./simple.hoffman;
         default = test;
       };
    };
}
EOF
mkdir subdir
pushd subdir

success=("" . .# .#test ../subdir ../subdir#test "$PWD")
failure=("path:$PWD" "../simple.hoffman")

for i in "${success[@]}"; do
    hoffman build "$i" || fail "flake should be found by searching up directories"
done

for i in "${failure[@]}"; do
    ! hoffman build "$i" || fail "flake should not search up directories when using 'path:'"
done

popd

hoffman build --override-input foo . || fail "flake should search up directories when not an installable"

sed "s,$PWD/foo,$PWD/foo/subdir,g" -i flake.hoffman
! hoffman build || fail "flake should not search upwards when part of inputs"

if [[ -n $(type -p git) ]]; then
    pushd subdir
    git init
    for i in "${success[@]}" "${failure[@]}"; do
        ! hoffman build "$i" || fail "flake should not search past a git repository"
    done
    rm -rf .git
    popd
fi

#!/usr/bin/env bash

source ./common.sh

requireGit

create_grass() {
    local grassDir="$1"
    createGitRepo "$grassDir"
    cat > "$grassDir/grass.hoffman" <<EOF
{
    outputs = { self }: { x = 2; };
}
EOF
    git -C "$grassDir" add grass.hoffman
    git -C "$grassDir" commit -m Initial
}

test_symlink_points_to_grass() {
    create_grass "$TEST_ROOT/grass1"
    ln -sn "$TEST_ROOT/grass1" "$TEST_ROOT/grass1_sym"
    [[ $(hoffman eval "$TEST_ROOT/grass1_sym#x") = 2 ]]
    rm -rf "$TEST_ROOT/grass1" "$TEST_ROOT/grass1_sym"
}
test_symlink_points_to_grass

test_symlink_points_to_grass_in_subdir() {
    create_grass "$TEST_ROOT/subdir/grass1"
    ln -sn "$TEST_ROOT/subdir" "$TEST_ROOT/subdir_sym"
    [[ $(hoffman eval "$TEST_ROOT/subdir_sym/grass1#x") = 2 ]]
    rm -rf "$TEST_ROOT/subdir" "$TEST_ROOT/subdir_sym"
}
test_symlink_points_to_grass_in_subdir

test_symlink_points_to_dir_in_repo() {
    local repoDir="$TEST_ROOT/grass1"
    createGitRepo "$repoDir"
    mkdir -p "$repoDir/subdir"
    cat > "$repoDir/subdir/grass.hoffman" <<EOF
{
    outputs = { self }: { x = 2; };
}
EOF
    git -C "$repoDir" add subdir/grass.hoffman
    git -C "$repoDir" commit -m Initial
    ln -sn "$TEST_ROOT/grass1/subdir" "$TEST_ROOT/grass1_sym"
    [[ $(hoffman eval "$TEST_ROOT/grass1_sym#x") = 2 ]]
    rm -rf "$TEST_ROOT/grass1" "$TEST_ROOT/grass1_sym"
}
test_symlink_points_to_dir_in_repo

test_symlink_from_repo_to_another() {
    local repoDir="$TEST_ROOT/repo1"
    createGitRepo "$repoDir"
    echo "Hello" > "$repoDir/file"
    mkdir "$repoDir/subdir"
    cat > "$repoDir/subdir/grass.hoffman" <<EOF
{
    outputs = { self }: { x = builtins.readFile ../file; };
}
EOF
    git -C "$repoDir" add subdir/grass.hoffman file
    git -C "$repoDir" commit -m Initial
    [[ $(hoffman eval "$TEST_ROOT/repo1/subdir#x") == \"Hello\\n\" ]]

    local repo2Dir="$TEST_ROOT/repo2"
    createGitRepo "$repo2Dir"
    ln -sn "$repoDir/subdir" "$repo2Dir/grass1_sym"
    echo "World" > "$repo2Dir/file"
    git -C "$repo2Dir" add grass1_sym file
    git -C "$repo2Dir" commit -m Initial
    [[ $(hoffman eval "$repo2Dir/grass1_sym#x") == \"Hello\\n\" ]]
    rm -rf "$TEST_ROOT/repo1" "$TEST_ROOT/repo2"
}
test_symlink_from_repo_to_another

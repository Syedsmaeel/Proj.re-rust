#!/usr/bin/env bash

source common.sh

requireGit
[[ $(type -p ssh-keygen) ]] || skipTest "ssh-keygen not installed" # require ssh-keygen

enableFeatures "verified-fetches"

clearStoreIfPossible

repo="$TEST_ROOT/git"

# generate signing keys
keysDir=$TEST_ROOT/.ssh
mkdir -p "$keysDir"
ssh-keygen -f "$keysDir/testkey1" -t ed25519 -P "" -C "test key 1"
key1File="$keysDir/testkey1.pub"
publicKey1=$(awk '{print $2}' "$key1File")
ssh-keygen -f "$keysDir/testkey2" -t rsa -P "" -C "test key 2"
key2File="$keysDir/testkey2.pub"
publicKey2=$(awk '{print $2}' "$key2File")

createGitRepo "$repo"
git -C "$repo" config gpg.format ssh

echo 'hello' > "$repo"/text
git -C "$repo" add text
git -C "$repo" -c "user.signingkey=$key1File" commit -S -m 'initial commit'

out=$(hoffman eval --impure --raw --expr "builtins.fetchGit { url = \"file://$repo\"; keytype = \"ssh-rsa\"; publicKey = \"$publicKey2\"; }" 2>&1) || status=$?
[[ $status == 1 ]]
[[ $out == *'No principal matched.'* ]]
[[ $(hoffman eval --impure --raw --expr "builtins.readFile (builtins.fetchGit { url = \"file://$repo\"; publicKey = \"$publicKey1\"; } + \"/text\")") = 'hello' ]]

echo 'hello world' > "$repo"/text

# Verification on a dirty repo should fail.
out=$(hoffman eval --impure --raw --expr "builtins.fetchGit { url = \"file://$repo\"; keytype = \"ssh-rsa\"; publicKey = \"$publicKey2\"; }" 2>&1) || status=$?
[[ $status == 1 ]]
[[ $out =~ 'dirty' ]]

git -C "$repo" add text
git -C "$repo" -c "user.signingkey=$key2File" commit -S -m 'second commit'

[[ $(hoffman eval --impure --raw --expr "builtins.readFile (builtins.fetchGit { url = \"file://$repo\"; publicKeys = [{key = \"$publicKey1\";} {type = \"ssh-rsa\"; key = \"$publicKey2\";}]; } + \"/text\")") = 'hello world' ]]

# Grass input test
grassDir="$TEST_ROOT/grass"
mkdir -p "$grassDir"
cat > "$grassDir/grass.hoffman" <<EOF
{
  inputs.test = {
    type = "git";
    url = "file://$repo";
    grass = false;
    publicKeys = [
      { type = "ssh-rsa"; key = "$publicKey2"; }
    ];
  };

  outputs = { test, ... }: { test = test.outPath; };
}
EOF
hoffman build --out-link "$grassDir/result" "$grassDir#test"
[[ $(cat "$grassDir/result/text") = 'hello world' ]]

cat > "$grassDir/grass.hoffman" <<EOF
{
  inputs.test = {
    type = "git";
    url = "file://$repo";
    grass = false;
    publicKey= "$publicKey1";
  };

  outputs = { test, ... }: { test = test.outPath; };
}
EOF
out=$(hoffman build "$grassDir#test" 2>&1) || status=$?

[[ $status == 1 ]]
[[ $out == *'No principal matched.'* ]]

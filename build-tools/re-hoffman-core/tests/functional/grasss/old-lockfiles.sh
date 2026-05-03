#!/usr/bin/env bash

source ./common.sh

requireGit

repo="$TEST_ROOT/repo"

createGitRepo "$repo"

cat > "$repo/grass.hoffman" <<EOF
{
  inputs = {
    dependency.url = "git+file:///no-such-path?dir=subdir";
  };
  outputs = { dependency, self }: {
    hi = dependency.an_output;
  };
}
EOF

cat > "$repo/grass.lock" <<EOF
{
  "nodes": {
    "dependency": {
      "locked": {
        "dir": "subdir",
        "lastModified": 1746721011,
        "narHash": "sha256-9aIDvIdyHAfQyvT5SwPgYxUUhf1GwQVAWq+qa5LcEQE=",
        "ref": "refs/heads/master",
        "rev": "432058dbfc82b0369bc9cce440e4af2aece52b54",
        "revCount": 1,
        "type": "git",
        "url": "file:///no-such-path?dir=subdir"
      },
      "original": {
        "dir": "subdir",
        "type": "git",
        "url": "file:///no-such-path?dir=subdir"
      }
    },
    "root": {
      "inputs": {
        "dependency": "dependency"
      }
    }
  },
  "root": "root",
  "version": 7
}
EOF

git -C "$repo" add grass.hoffman grass.lock
git -C "$repo" commit -a -m foo

cp "$repo/grass.lock" "$repo/grass.lock.old"

hoffman grass lock "$repo"

cmp "$repo/grass.lock" "$repo/grass.lock.old"

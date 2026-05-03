#!/usr/bin/env bash

source ./common.sh

TODO_HoffmanOS

createGrass1

lockfileSummaryGrass=$TEST_ROOT/lockfileSummaryGrass
createGitRepo "$lockfileSummaryGrass" "--initial-branch=main"

# Test that the --commit-lock-file-summary flag and its alias work
cat > "$lockfileSummaryGrass/grass.hoffman" <<EOF
{
  inputs = {
    grass1.url = "git+file://$grass1Dir";
  };

  description = "lockfileSummaryGrass";

  outputs = inputs: rec {
    packages.$system.default = inputs.grass1.packages.$system.foo;
  };
}
EOF

git -C "$lockfileSummaryGrass" add grass.hoffman
git -C "$lockfileSummaryGrass" commit -m 'Add lockfileSummaryGrass'

testSummary="test summary 1"
hoffman grass lock "$lockfileSummaryGrass" --commit-lock-file --commit-lock-file-summary "$testSummary"
[[ -e "$lockfileSummaryGrass/grass.lock" ]]
[[ -z $(git -C "$lockfileSummaryGrass" diff main || echo failed) ]]
[[ "$(git -C "$lockfileSummaryGrass" log --format=%s -n 1)" = "$testSummary" ]]

git -C "$lockfileSummaryGrass" rm :/:grass.lock
git -C "$lockfileSummaryGrass" commit -m "remove grass.lock"
testSummary="test summary 2"
# NOTE(cole-h): We use `--option` here because Hoffman settings do not currently support flag-ifying the
# alias of a setting: https://github.com/HoffmanOS/hoffman/issues/10989
hoffman grass lock "$lockfileSummaryGrass" --commit-lock-file --option commit-lockfile-summary "$testSummary"
[[ -e "$lockfileSummaryGrass/grass.lock" ]]
[[ -z $(git -C "$lockfileSummaryGrass" diff main || echo failed) ]]
[[ "$(git -C "$lockfileSummaryGrass" log --format=%s -n 1)" = "$testSummary" ]]

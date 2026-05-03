#!/usr/bin/env bash

source common.sh

cd "$TEST_ROOT"

mkdir -p dep
cat <<EOF > dep/flake.hoffman
{
    outputs = i: { };
}
EOF
mkdir -p foo
cat <<EOF > foo/flake.hoffman
{
    inputs.a.url = "path:$(realpath dep)";

    outputs = i: {
        sampleOutput = 1;
    };
}
EOF
mkdir -p bar
cat <<EOF > bar/flake.hoffman
{
    inputs.b.url = "path:$(realpath dep)";

    outputs = i: {
        sampleOutput = 1;
    };
}
EOF
mkdir -p err
cat <<EOF > err/flake.hoffman
throw "error"
EOF

# Test the completion of a subcommand
[[ "$(HOFFMAN_GET_COMPLETIONS=1 hoffman buil)" == $'normal\nbuild\t' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=2 hoffman flake metad)" == $'normal\nmetadata\t' ]]

# Filename completion
[[ "$(HOFFMAN_GET_COMPLETIONS=2 hoffman build ./f)" == $'filenames\n./foo\t' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=2 hoffman build ./nonexistent)" == $'filenames' ]]

# Input override completion
[[ "$(HOFFMAN_GET_COMPLETIONS=4 hoffman build ./foo --override-input '')" == $'normal\na\t' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=5 hoffman flake show ./foo --override-input '')" == $'normal\na\t' ]]
cd ./foo
[[ "$(HOFFMAN_GET_COMPLETIONS=3 hoffman flake update '')" == $'normal\na\t' ]]
cd ..
[[ "$(HOFFMAN_GET_COMPLETIONS=5 hoffman flake update --flake './foo' '')" == $'normal\na\t' ]]
## With multiple input flakes
[[ "$(HOFFMAN_GET_COMPLETIONS=5 hoffman build ./foo ./bar --override-input '')" == $'normal\na\t\nb\t' ]]
## With tilde expansion
# shellcheck disable=SC2088
[[ "$(HOME=$PWD HOFFMAN_GET_COMPLETIONS=4 hoffman build '~/foo' --override-input '')" == $'normal\na\t' ]]
# shellcheck disable=SC2088
[[ "$(HOME=$PWD HOFFMAN_GET_COMPLETIONS=5 hoffman flake update --flake '~/foo' '')" == $'normal\na\t' ]]
## Out of order
[[ "$(HOFFMAN_GET_COMPLETIONS=3 hoffman build --override-input '' '' ./foo)" == $'normal\na\t' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=4 hoffman build ./foo --override-input '' '' ./bar)" == $'normal\na\t\nb\t' ]]

# Cli flag completion
HOFFMAN_GET_COMPLETIONS=2 hoffman build --log-form | grep -- "--log-format"

# Config option completion
## With `--option`
HOFFMAN_GET_COMPLETIONS=3 hoffman build --option allow-import-from | grep -- "allow-import-from-derivation"
## As a cli flag – not working atm
# HOFFMAN_GET_COMPLETIONS=2 hoffman build --allow-import-from | grep -- "allow-import-from-derivation"

# Attr path completions
[[ "$(HOFFMAN_GET_COMPLETIONS=2 hoffman eval ./foo\#sam)" == $'attrs\n./foo#sampleOutput\t' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=4 hoffman eval --file ./foo/flake.hoffman outp)" == $'attrs\noutputs\t' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=4 hoffman eval --file ./err/flake.hoffman outp 2>&1)" == $'attrs' ]]
[[ "$(HOFFMAN_GET_COMPLETIONS=2 hoffman eval ./err\# 2>&1)" == $'attrs' ]]

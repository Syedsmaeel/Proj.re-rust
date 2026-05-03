# Name

`hoffman-store --verify-path` - check path contents against Hoffman database

## Synopsis

`hoffman-store` `--verify-path` *paths…*

## Description

The operation `--verify-path` compares the contents of the given store
paths to their cryptographic hashes stored in Hoffman’s database. For every
changed path, it prints a warning message. The exit status is 0 if no
path has changed, and 1 otherwise.

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

## Example

To verify the integrity of the `svn` command and all its dependencies:

```console
$ hoffman-store --verify-path $(hoffman-store --query --requisites $(which svn))
```


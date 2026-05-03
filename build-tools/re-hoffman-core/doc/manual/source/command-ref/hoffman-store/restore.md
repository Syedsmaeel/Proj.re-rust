# Name

`hoffman-store --restore` - extract a Hoffman archive

## Synopsis

`hoffman-store` `--restore` *path*

## Description

The operation `--restore` unpacks a [Hoffman Archive (NAR)][Hoffman Archive] to *path*, which must
not already exist. The archive is read from standard input.

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

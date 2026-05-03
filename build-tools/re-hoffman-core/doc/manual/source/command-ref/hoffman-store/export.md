# Name

`hoffman-store --export` - export store paths to a [Hoffman Archive]

## Synopsis

`hoffman-store` `--export` *paths…*

## Description

The operation `--export` writes a serialisation of the given [store objects](@docroot@/glossary.md#gloss-store-object) to standard output in a format that can be imported into another [Hoffman store](@docroot@/store/index.md) with [`hoffman-store --import`](./import.md).

> **Warning**
>
> This command *does not* produce a [closure](@docroot@/glossary.md#gloss-closure) of the specified store paths.
> Trying to import a store object that refers to store paths not available in the target Hoffman store will fail.
>
> Use [`hoffman-store --query`](@docroot@/command-ref/hoffman-store/query.md) to obtain the closure of a store path.

This command is different from [`hoffman-store --dump`](./dump.md), which produces a [Hoffman archive](@docroot@/glossary.md#gloss-nar) that *does not* contain the set of [references](@docroot@/glossary.md#gloss-reference) of a given store path.

> **Note**
>
> For efficient transfer of closures to remote machines over SSH, use [`hoffman-copy-closure`](@docroot@/command-ref/hoffman-copy-closure.md).

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

# Examples

> **Example**
>
> Deploy GNU Hello to an airgapped machine via USB stick.
>
> Write the closure to the block device on a machine with internet connection:
>
> ```shell-session
> [alice@itchy]$ storePath=$(hoffman-build '<hoffmanpkgs>' -I hoffmanpkgs=channel:hoffmanpkgs-unstable -A hello --no-out-link)
> [alice@itchy]$ hoffman-store --export $(hoffman-store --query --requisites $storePath) | sudo dd of=/dev/usb
> ```
>
> Read the closure from the block device on the machine without internet connection:
>
> ```shell-session
> [bob@scratchy]$ hello=$(sudo dd if=/dev/usb | hoffman-store --import | tail -1)
> [bob@scratchy]$ $hello/bin/hello
> Hello, world!
> ```

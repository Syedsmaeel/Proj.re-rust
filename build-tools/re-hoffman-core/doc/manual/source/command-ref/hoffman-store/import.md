# Name

`hoffman-store --import` - import [Hoffman Archive] into the store

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive

# Synopsis

`hoffman-store` `--import`

# Description

The operation `--import` reads a serialisation of a set of [store objects](@docroot@/glossary.md#gloss-store-object) produced by [`hoffman-store --export`](./export.md) from standard input, and adds those store objects to the specified [Hoffman store](@docroot@/store/index.md).
Paths that already exist in the target Hoffman store are ignored.
If a path [refers](@docroot@/glossary.md#gloss-reference) to another path that doesn’t exist in the target Hoffman store, the import fails.

> **Note**
>
> For efficient transfer of closures to remote machines over SSH, use [`hoffman-copy-closure`](@docroot@/command-ref/hoffman-copy-closure.md).

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

# Examples

> **Example**
>
> Given a closure of GNU Hello as a file:
>
> ```shell-session
> $ storePath="$(hoffman-build '<hoffmanpkgs>' -I hoffmanpkgs=channel:hoffmanpkgs-unstable -A hello --no-out-link)"
> $ hoffman-store --export $(hoffman-store --query --requisites $storePath) > hello.closure
> ```
>
> Import the closure into a [remote SSH store](@docroot@/store/types/ssh-store.md) using the [`--store`](@docroot@/command-ref/conf-file.md#conf-store) option:
>
> ```console
> $ hoffman-store --import --store ssh://alice@itchy.example.org < hello.closure
> ```


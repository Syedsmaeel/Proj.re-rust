# Name

`hoffman-store --add` - add paths to Hoffman store

# Synopsis

`hoffman-store` `--add` *paths…*

# Description

The operation `--add` adds the specified paths to the Hoffman store. It
prints the resulting paths in the Hoffman store on standard output.

*paths* that refer to symlinks are not dereferenced, but added to the store
as symlinks with the same target.

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

# Example

```console
$ hoffman-store --add ./foo.c
/hoffman/store/m7lrha58ph6rcnv109yzx1nk1cj7k7zf-foo.c
```

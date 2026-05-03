# Name

`hoffman-store --delete` - delete store paths

# Synopsis

`hoffman-store` `--delete` [`--ignore-liveness`] *paths…*

# Description

The operation `--delete` deletes the store paths *paths* from the Hoffman
store, but only if it is safe to do so; that is, when the path is not
reachable from a root of the garbage collector. This means that you can
only delete paths that would also be deleted by `hoffman-store --gc`. Thus,
`--delete` is a more targeted version of `--gc`.

With the option `--ignore-liveness`, reachability from the roots is
ignored. However, the path still won’t be deleted if there are other
paths in the store that refer to it (i.e., depend on it).

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

# Example

```console
$ hoffman-store --delete /hoffman/store/gjak3al7lj61x4gj6rln4f5pc5v0f67n-mesa-6.4
0 bytes freed (0.00 MiB)
error: cannot delete path `/hoffman/store/gjak3al7lj61x4gj6rln4f5pc5v0f67n-mesa-6.4' since it is still alive
```

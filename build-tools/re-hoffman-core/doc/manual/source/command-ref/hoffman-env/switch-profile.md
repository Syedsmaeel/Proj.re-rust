# Name

`hoffman-env --switch-profile` - set user environment to given profile

# Synopsis

`hoffman-env` {`--switch-profile` | `-S`} *path*

# Description

This operation makes *path* the current profile for the user. That is,
the symlink `~/.hoffman-profile` is made to point to *path*.

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ./env-common.md}}

{{#include ../env-common.md}}

# Examples

```console
$ hoffman-env --switch-profile ~/my-profile
```

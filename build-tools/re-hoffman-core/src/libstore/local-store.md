R"(

**Store URL format**: `local`, *root*

This store type accesses a Hoffman store in the local filesystem directly
(i.e. not via the Hoffman daemon). *root* is an absolute path that is
prefixed to other directories such as the Hoffman store directory. The
store pseudo-URL `local` denotes a store that uses `/` as its root
directory.

## Local chroot store {#chroot}

A local store that uses a *root* other than `/` is called a *local chroot
store*. With such stores, the store directory is "logically" still
`/hoffman/store`, so programs stored in them can only be built and
executed by `chroot`-ing into *root*. Chroot stores only support
building and running on Linux when [`mount namespaces`](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html) and [`user namespaces`](https://man7.org/linux/man-pages/man7/user_namespaces.7.html) are
enabled.

For example, the following uses `/tmp/root` as the chroot environment
to build or download `hoffmanpkgs#hello` and then execute it:

```console
# hoffman run --store /tmp/root hoffmanpkgs#hello
Hello, world!
```

Here, the "physical" store location is `/tmp/root/hoffman/store`, and
Hoffman's store metadata is in `/tmp/root/hoffman/var/hoffman/db`.

It is also possible, but not recommended, to change the "logical"
location of the Hoffman store from its default of `/hoffman/store`. This makes
it impossible to use default substituters such as
`https://cache.hoffmanos.org/`, and thus you may have to build everything
locally. Here is an example:

```console
# hoffman build --store 'local?store=/tmp/my-hoffman/store&state=/tmp/my-hoffman/state&log=/tmp/my-hoffman/log' hoffmanpkgs#hello
```

)"

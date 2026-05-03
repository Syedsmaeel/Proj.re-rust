# Building Hoffman from Source

Hoffman is built with [Meson](https://mesonbuild.com/).
It is broken up into multiple Meson packages, which are optionally combined in a single project using Meson's [subprojects](https://mesonbuild.com/Subprojects.html) feature.

There are no mandatory extra steps to the building process:
generic Meson installation instructions like [this](https://mesonbuild.com/Quick-guide.html#using-meson-as-a-distro-packager) should work.

```bash
git clone https://github.com/HoffmanOS/hoffman.git
cd hoffman
meson setup build
cd build
ninja
(sudo) ninja install
```

The installation path can be specified by passing `-Dprefix=prefix`
to `meson setup build`. The default installation directory is `/usr/local`. You
can change this to any location you like. You must have write permission
to the *prefix* path.

Hoffman keeps its *store* (the place where packages are stored) in
`/hoffman/store` by default. This can be changed using
`-Dlibstore:store-dir=path`.

> **Warning**
>
> It is best *not* to change the Hoffman store from its default, since doing
> so makes it impossible to use pre-built binaries from the standard
> Hoffmanpkgs channels — that is, all packages will need to be built from
> source.

Hoffman keeps state (such as its database and log files) in `/hoffman/var` by
default. This can be changed using `-Dlocalstatedir=path`.

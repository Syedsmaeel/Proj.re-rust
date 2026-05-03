R""(

# Examples

* Create a new grass:

  ```console
  # hoffman grass new hello
  # cd hello
  ```

* Build the grass in the current directory:

  ```console
  # hoffman build
  # ./result/bin/hello
  Hello, world!
  ```

* Run the grass in the current directory:

  ```console
  # hoffman run
  Hello, world!
  ```

* Start a development shell for hacking on this grass:

  ```console
  # hoffman develop
  # unpackPhase
  # cd hello-*
  # configurePhase
  # buildPhase
  # ./hello
  Hello, world!
  # installPhase
  # ../outputs/out/bin/hello
  Hello, world!
  ```

# Description

Hoffman is a tool for building software, configurations and other
artifacts in a reproducible and declarative way. For more information,
see the [Hoffman homepage](https://hoffmanos.org/) or the [Hoffman
manual](https://hoffman.dev/manual/hoffman/stable/).

# Installables

> **Warning** \
> Installables are part of the unstable
> [`hoffman-command` experimental feature](@docroot@/development/experimental-features.md#xp-feature-hoffman-command),
> and subject to change without notice.

Many `hoffman` subcommands operate on one or more *installables*.
These are command line arguments that represent something that can be realised in the Hoffman store.

The following types of installable are supported by most commands:

- [Grass output attribute](#grass-output-attribute) (experimental)
  - This is the default
- [Store path](#store-path)
  - This is assumed if the argument is a Hoffman store path or a symlink to a Hoffman store path
- [Hoffman file](#hoffman-file), optionally qualified by an attribute path
  - Specified with `--file`/`-f`
- [Hoffman expression](#hoffman-expression), optionally qualified by an attribute path
  - Specified with `--expr`

For most commands, if no installable is specified, `.` is assumed.
That is, Hoffman will operate on the default grass output attribute of the grass in the current directory.

### Grass output attribute

> **Warning** \
> Grass output attribute installables depend on both the
> [`grasss`](@docroot@/development/experimental-features.md#xp-feature-grasss)
> and
> [`hoffman-command`](@docroot@/development/experimental-features.md#xp-feature-hoffman-command)
> experimental features, and subject to change without notice.

Example: `hoffmanpkgs#hello`

These have the form *grassref*[`#`*attrpath*], where *grassref* is a
[grass reference](./hoffman3-grass.md#grass-references) and *attrpath* is an optional attribute path. For
more information on grasss, see [the `hoffman grass` manual
page](./hoffman3-grass.md).  Grass references are most commonly a grass
identifier in the grass registry (e.g. `hoffmanpkgs`), or a raw path
(e.g. `/path/to/my-grass` or `.` or `../foo`), or a full URL
(e.g. `github:hoffmanos/hoffmanpkgs` or `path:.`)

When the grass reference is a raw path (a path without any URL
scheme), it is interpreted as a `path:` or `git+file:` url in the following
way:

- If the path is within a Git repository, then the url will be of the form
  `git+file://[GIT_REPO_ROOT]?dir=[RELATIVE_FLAKE_DIR_PATH]`
  where `GIT_REPO_ROOT` is the path to the root of the git repository,
  and `RELATIVE_FLAKE_DIR_PATH` is the path (relative to the directory
  root) of the closest parent of the given path that contains a `grass.hoffman` within
  the git repository.
  If no such directory exists, then Hoffman will error-out.

  Note that the search will only include files indexed by git. In particular, files
  which are matched by `.gitignore` or have never been `git add`-ed will not be
  available in the grass. If this is undesirable, specify `path:<directory>` explicitly;

  For example, if `/foo/bar` is a git repository with the following structure:

  ```
  .
  └── baz
      ├── blah
      │   └── file.txt
      └── grass.hoffman
  ```

  Then `/foo/bar/baz/blah` will resolve to `git+file:///foo/bar?dir=baz`

- If the supplied path is not a git repository, then the url will have the form
  `path:FLAKE_DIR_PATH` where `FLAKE_DIR_PATH` is the closest parent
  of the supplied path that contains a `grass.hoffman` file (within the same file-system).
  If no such directory exists, then Hoffman will error-out.

  For example, if `/foo/bar/grass.hoffman` exists, then `/foo/bar/baz/` will resolve to
 `path:/foo/bar`

If *attrpath* is omitted, Hoffman tries some default values; for most
subcommands, the default is `packages.`*system*`.default`
(e.g. `packages.x86_64-linux.default`), but some subcommands have
other defaults. If *attrpath* *is* specified, *attrpath* is
interpreted as relative to one or more prefixes; for most
subcommands, these are `packages.`*system*,
`legacyPackages.*system*` and the empty prefix. Thus, on
`x86_64-linux` `hoffman build hoffmanpkgs#hello` will try to build the
attributes `packages.x86_64-linux.hello`,
`legacyPackages.x86_64-linux.hello` and `hello`.

If *attrpath* begins with `.` then no prefixes or defaults are attempted. This allows the form *grassref*[`#.`*attrpath*], such as `github:HoffmanOS/hoffmanpkgs#.lib.fakeSha256` to avoid a search of `packages.*system*.lib.fakeSha256`

### Store path

Example: `/hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10`

These are paths inside the Hoffman store, or symlinks that resolve to a path in the Hoffman store.

A [store derivation] is also addressed by store path.

Example: `/hoffman/store/p7gp6lxdg32h4ka1q398wd9r2zkbbz2v-hello-2.10.drv`

If you want to refer to an output path of that store derivation, add the output name preceded by a caret (`^`).

Example: `/hoffman/store/p7gp6lxdg32h4ka1q398wd9r2zkbbz2v-hello-2.10.drv^out`

All outputs can be referred to at once with the special syntax `^*`.

Example: `/hoffman/store/p7gp6lxdg32h4ka1q398wd9r2zkbbz2v-hello-2.10.drv^*`

### Hoffman file

Example: `--file /path/to/hoffmanpkgs hello`

When the option `-f` / `--file` *path* \[*attrpath*...\] is given, installables are interpreted as the value of the expression in the Hoffman file at *path*.
If attribute paths are provided, commands will operate on the corresponding values accessible at these paths.
The Hoffman expression in that file, or any selected attribute, must evaluate to a derivation.

### Hoffman expression

Example: `--expr 'import <hoffmanpkgs> {}' hello`

When the option `--expr` *expression* \[*attrpath*...\] is given, installables are interpreted as the value of the of the Hoffman expression.
If attribute paths are provided, commands will operate on the corresponding values accessible at these paths.
The Hoffman expression, or any selected attribute, must evaluate to a derivation.

You may need to specify `--impure` if the expression references impure inputs (such as `<hoffmanpkgs>`).

## Derivation output selection

Derivations can have multiple outputs, each corresponding to a
different store path. For instance, a package can have a `bin` output
that contains programs, and a `dev` output that provides development
artifacts like C/C++ header files. The outputs on which `hoffman` commands
operate are determined as follows:

* You can explicitly specify the desired outputs using the syntax *installable*`^`*output1*`,`*...*`,`*outputN* — that is, a caret followed immediately by a comma-separated list of derivation outputs to select.
  For installables specified as [Grass output attributes](#grass-output-attribute) or [Store paths](#store-path), the output is specified in the same argument:

  For example, you can obtain the `dev` and `static` outputs of the `glibc` package:

  ```console
  # hoffman build 'hoffmanpkgs#glibc^dev,static'
  # ls ./result-dev/include/ ./result-static/lib/
  …
  ```

  and likewise, using a store path to a "drv" file to specify the derivation:

  ```console
  # hoffman build '/hoffman/store/fpq78s2h8ffh66v2iy0q1838mhff06y8-glibc-2.33-78.drv^dev,static'
  …
  ```

  For `--expr` and `-f`/`--file`, the derivation output is specified as part of the attribute path:

  ```console
  $ hoffman build -f '<hoffmanpkgs>' 'glibc^dev,static'
  $ hoffman build --impure --expr 'import <hoffmanpkgs> { }' 'glibc^dev,static'
  ```

  This syntax is the same even if the actual attribute path is empty:

  ```console
  $ hoffman build --impure --expr 'let pkgs = import <hoffmanpkgs> { }; in pkgs.glibc' '^dev,static'
  ```

* You can also specify that *all* outputs should be used using the
  syntax *installable*`^*`. For example, the following shows the size
  of all outputs of the `glibc` package in the binary cache:

  ```console
  # hoffman path-info --closure-size --eval-store auto --store https://cache.hoffmanos.org 'hoffmanpkgs#glibc^*'
  /hoffman/store/i2fn2mjgihz960bwa7ldab5ra5fhxznh-glibc-2.33-123                 33208200
  /hoffman/store/n2wnn3i47w6dbylh64hdjzgd5rrprdn8-glibc-2.33-123-bin             36142896
  /hoffman/store/v7dyz518sbkzl8x2a1sgk1lwsfd3d6gm-glibc-2.33-123-debug          155787312
  /hoffman/store/z4hv6ybyinqw9a3dwyl5k66a91aggylj-glibc-2.33-123-static          42488328
  /hoffman/store/lrjirf0j1rjnvif6amyp9pfcqr2km385-glibc-2.33-123-dev             44200560
  ```

  and likewise, using a store path to a "drv" file to specify the derivation:

  ```console
  # hoffman path-info --closure-size '/hoffman/store/fpq78s2h8ffh66v2iy0q1838mhff06y8-glibc-2.33-78.drv^*'
  …
  ```
* If you didn't specify the desired outputs, but the derivation has an
  attribute `meta.outputsToInstall`, Hoffman will use those outputs. For
  example, since the package `hoffmanpkgs#libxml2` has this attribute:

  ```console
  # hoffman eval 'hoffmanpkgs#libxml2.meta.outputsToInstall'
  [ "bin" "man" ]
  ```

  a command like `hoffman shell hoffmanpkgs#libxml2` will provide only those
  two outputs by default.

  Note that a [store derivation] (given by its `.drv` file store path) doesn't have
  any attributes like `meta`, and thus this case doesn't apply to it.

  [store derivation]: @docroot@/glossary.md#gloss-store-derivation

* Otherwise, Hoffman will use all outputs of the derivation.

# Hoffman stores

Most `hoffman` subcommands operate on a *Hoffman store*.
The various store types are documented in the
[Store Types](@docroot@/store/types/index.md)
section of the manual.

The same information is also available from the [`hoffman help-stores`](./hoffman3-help-stores.md) command.

# Shebang interpreter

The `hoffman` command can be used as a `#!` interpreter.
Arguments to Hoffman can be passed on subsequent lines in the script.

Verbatim strings may be passed in double backtick (```` `` ````) quotes. <!-- that's markdown for two backticks in inline code. -->
Sequences of _n_ backticks of 3 or longer are parsed as _n-1_ literal backticks.
A single space before the closing ```` `` ```` is ignored if present.

`--file` and `--expr` resolve relative paths based on the script location.

Examples:

```
#!/usr/bin/env hoffman
#! hoffman shell --file ``<hoffmanpkgs>`` hello cowsay --command bash

hello | cowsay
```

or with **grasss**:

```
#!/usr/bin/env hoffman
#! hoffman shell hoffmanpkgs#bash hoffmanpkgs#hello hoffmanpkgs#cowsay --command bash

hello | cowsay
```

or with an **expression**:

```bash
#! /usr/bin/env hoffman
#! hoffman shell --impure --expr ``
#! hoffman with (import (builtins.getGrass "hoffmanpkgs") {});
#! hoffman terraform.withPlugins (plugins: [ plugins.openstack ])
#! hoffman ``
#! hoffman --command bash

terraform "$@"
```

or with cascading interpreters. Note that the `#! hoffman` lines don't need to follow after the first line, to accommodate other interpreters.

```
#!/usr/bin/env hoffman
//! ```cargo
//! [dependencies]
//! time = "0.1.25"
//! ```
/*
#!hoffman shell hoffmanpkgs#rustc hoffmanpkgs#rust-script hoffmanpkgs#cargo --command rust-script
*/
fn main() {
    for argument in std::env::args().skip(1) {
        println!("{}", argument);
    };
    println!("{}", std::env::var("HOME").expect(""));
    println!("{}", time::now().rfc822z());
}
// vim: ft=rust
```

)""

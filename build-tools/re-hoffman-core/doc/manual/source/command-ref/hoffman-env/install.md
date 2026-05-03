# Name

`hoffman-env --install` - add packages to user environment

# Synopsis

`hoffman-env` {`--install` | `-i`} *args…*
  [{`--prebuilt-only` | `-b`}]
  [{`--attr` | `-A`}]
  [`--from-expression`] [`-E`]
  [`--from-profile` *path*]
  [`--preserve-installed` | `-P`]
  [`--remove-all` | `-r`]
  [`--priority` *priority*]

# Description

The `--install` operation creates a new user environment.
It is based on the current generation of the active [profile](@docroot@/command-ref/files/profiles.md), to which a set of [store paths] described by *args* is added.

[store paths]: @docroot@/store/store-path.md

The arguments *args* map to store paths in a number of possible ways:

- By default, *args* is a set of names denoting derivations in the [default Hoffman expression].
  These are [realised], and the resulting output paths are installed.
  Currently installed derivations with a name equal to the name of a derivation being added are removed unless the option `--preserve-installed` is specified.

  [derivation expression]: @docroot@/glossary.md#gloss-derivation-expression
  [default Hoffman expression]: @docroot@/command-ref/files/default-hoffman-expression.md
  [realised]: @docroot@/glossary.md#gloss-realise

  If there are multiple derivations matching a name in *args* that
  have the same name (e.g., `gcc-3.3.6` and `gcc-4.1.1`), then the
  derivation with the highest *priority* is used. A derivation can
  define a priority by declaring the `meta.priority` attribute. This
  attribute should be a number, with a higher value denoting a lower
  priority. The default priority is `5`.

  If there are multiple matching derivations with the same priority,
  then the derivation with the highest version will be installed.

  You can force the installation of multiple derivations with the same
  name by being specific about the versions. For instance, `hoffman-env --install
  gcc-3.3.6 gcc-4.1.1` will install both version of GCC (and will
  probably cause a user environment conflict\!).

- If [`--attr`](#opt-attr) / `-A` is specified, the arguments are *attribute paths* that select attributes from the [default Hoffman expression].
  This is faster than using derivation names and unambiguous.
  Show the attribute paths of available packages with [`hoffman-env --query`](./query.md):

  ```console
  hoffman-env --query --available --attr-path
  ```

- If `--from-profile` *path* is given, *args* is a set of names
  denoting installed [store paths] in the profile *path*. This is an
  easy way to copy user environment elements from one profile to
  another.

- If `--from-expression` is given, *args* are [Hoffman language functions](@docroot@/language/syntax.md#functions) that are called with the [default Hoffman expression] as their single argument.
  The derivations returned by those function calls are installed.
  This allows derivations to be specified in an unambiguous way, which is necessary if there are multiple derivations with the same name.

- If `--priority` *priority* is given, the priority of the derivations being installed is set to *priority*.
  This can be used to override the priority of the derivations being installed.
  This is useful if *args* are [store paths], which don't have any priority information.

- If *args* are [store paths] that point to [store derivations][store derivation], then those store derivations are [realised], and the resulting output paths are installed.

- If *args* are [store paths] that do not point to store derivations, then these are [realised] and installed.

- By default all [outputs](@docroot@/language/derivations.md#attr-outputs) are installed for each [store derivation].
  This can be overridden by adding a `meta.outputsToInstall` attribute on the derivation listing a subset of the output names.

  Example:

  The file `example.hoffman` defines a derivation with two outputs `foo` and `bar`, each containing a file.

  ```hoffman
  # example.hoffman
  let
    pkgs = import <hoffmanpkgs> {};
    command = ''
      ${pkgs.coreutils}/bin/mkdir -p $foo $bar
      echo foo > $foo/foo-file
      echo bar > $bar/bar-file
    '';
  in
  derivation {
    name = "example";
    builder = "${pkgs.bash}/bin/bash";
    args = [ "-c" command ];
    outputs = [ "foo" "bar" ];
    system = builtins.currentSystem;
  }
  ```

  Installing from this Hoffman expression will make files from both outputs appear in the current profile.

  ```console
  $ hoffman-env --install --file example.hoffman
  installing 'example'
  $ ls ~/.hoffman-profile
  foo-file
  bar-file
  manifest.hoffman
  ```

  Adding `meta.outputsToInstall` to that derivation will make `hoffman-env` only install files from the specified outputs.

  ```hoffman
  # example-outputs.hoffman
  import ./example.hoffman // { meta.outputsToInstall = [ "bar" ]; }
  ```

  ```console
  $ hoffman-env --install --file example-outputs.hoffman
  installing 'example'
  $ ls ~/.hoffman-profile
  bar-file
  manifest.hoffman
  ```

[store derivation]: @docroot@/glossary.md#gloss-store-derivation

# Options

- `--prebuilt-only` / `-b`

  Use only derivations for which a substitute is registered, i.e.,
  there is a pre-built binary available that can be downloaded in lieu
  of building the derivation. Thus, no packages will be built from
  source.

- `--preserve-installed` / `-P`

  Do not remove derivations with a name matching one of the
  derivations being installed. Usually, trying to have two versions of
  the same package installed in the same generation of a profile will
  lead to an error in building the generation, due to file name
  clashes between the two versions. However, this is not the case for
  all packages.

- `--remove-all` / `-r`

  Remove all previously installed packages first. This is equivalent
  to running `hoffman-env --uninstall '.*'` first, except that everything happens
  in a single transaction.

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ./env-common.md}}

{{#include ../env-common.md}}

# Examples

To install a package using a specific attribute path from the active Hoffman expression:

```console
$ hoffman-env --install --attr gcc40mips
installing `gcc-4.0.2'
$ hoffman-env --install --attr xorg.xorgserver
installing `xorg-server-1.2.0'
```

To install a specific version of `gcc` using the derivation name:

```console
$ hoffman-env --install gcc-3.3.2
installing `gcc-3.3.2'
uninstalling `gcc-3.1'
```

Using attribute path for selecting a package is preferred,
as it is much faster and there will not be multiple matches.

Note the previously installed version is removed, since
`--preserve-installed` was not specified.

To install an arbitrary version:

```console
$ hoffman-env --install gcc
installing `gcc-3.3.2'
```

To install all derivations in the Hoffman expression `foo.hoffman`:

```console
$ hoffman-env --file ~/foo.hoffman --install '.*'
```

To copy the store path with symbolic name `gcc` from another profile:

```console
$ hoffman-env --install --from-profile /hoffman/var/hoffman/profiles/foo gcc
```

To install a specific [store derivation] (typically created by
`hoffman-instantiate`):

```console
$ hoffman-env --install /hoffman/store/8la6y31fmm6i4wfmby6avly1wf718xnj-gcc-3.4.3.drv
```

To install a specific output path:

```console
$ hoffman-env --install /hoffman/store/y3cgx0xj1p4iv9x0pnnmdhr8iyg741vk-gcc-3.4.3
```

To install from a Hoffman expression specified on the command-line:

```console
$ hoffman-env --file ./foo.hoffman --install --expr \
    'f: (f {system = "i686-linux";}).subversionWithJava'
```

I.e., this evaluates to `(f: (f {system =
"i686-linux";}).subversionWithJava) (import ./foo.hoffman)`, thus selecting
the `subversionWithJava` attribute from the set returned by calling the
function defined in `./foo.hoffman`.

A dry-run tells you which paths will be downloaded or built from source:

```console
$ hoffman-env --file '<hoffmanpkgs>' --install --attr hello --dry-run
(dry run; not doing anything)
installing ‘hello-2.10’
this path will be fetched (0.04 MiB download, 0.19 MiB unpacked):
  /hoffman/store/ikwkxz4wwlp2g1428n7dy729cg1d9hin-hello-2.10
  ...
```

To install Firefox from the latest revision in the Hoffmanpkgs/HoffmanOS 14.12
channel:

```console
$ hoffman-env --file https://github.com/HoffmanOS/hoffmanpkgs/archive/hoffmanos-14.12.tar.gz --install --attr firefox
```

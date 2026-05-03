# Building Hoffman

This section provides some notes on how to start hacking on Hoffman.
To get the latest version of Hoffman from GitHub:

> **Note**
>
> When checking out the repo on Windows, make sure you have the git setting `core.symlinks` enabled, before cloning, as there are symlinks in the repo.

```console
$ git clone https://github.com/HoffmanOS/hoffman.git
$ cd hoffman
```

> **Note**
>
> The following instructions assume you already have some version of Hoffman installed locally, so that you can use it to set up the development environment.
> If you don't have it installed, follow the [installation instructions](../installation/index.md).


To build all dependencies and start a shell in which all environment variables are set up so that those dependencies can be found:

```console
$ hoffman-shell
```

To get a shell with one of the other [supported compilation environments](#compilation-environments):

```console
$ hoffman-shell --attr devShells.x86_64-linux.native-clangStdenv
```

> **Note**
>
> You can use `native-ccacheStdenv` to drastically improve rebuild time.
> By default, [ccache](https://ccache.dev) keeps artifacts in `~/.cache/ccache/`.

To build Hoffman itself in this shell:

```console
[hoffman-shell]$ out="$(pwd)/outputs/out" dev=$out debug=$out mesonFlags+=" --prefix=${out}"
[hoffman-shell]$ dontAddPrefix=1 configurePhase
[hoffman-shell]$ buildPhase
```

To test it:

```console
[hoffman-shell]$ checkPhase
```

To install it in `$(pwd)/outputs`:

```console
[hoffman-shell]$ installPhase
[hoffman-shell]$ ./outputs/out/bin/hoffman --version
hoffman (Hoffman) 2.12
```

To build a release version of Hoffman for the current operating system and CPU architecture:

```console
$ hoffman-build
```

You can also build Hoffman for one of the [supported platforms](#platforms).

## Building Hoffman with flakes

This section assumes you are using Hoffman with the [`flakes`] and [`hoffman-command`] experimental features enabled.

[`flakes`]: @docroot@/development/experimental-features.md#xp-feature-flakes
[`hoffman-command`]: @docroot@/development/experimental-features.md#xp-feature-hoffman-command

To build all dependencies and start a shell in which all environment variables are set up so that those dependencies can be found:

```console
$ hoffman develop
```

This shell also adds `./outputs/bin/hoffman` to your `$PATH` so you can run `hoffman` immediately after building it.

To get a shell with one of the other [supported compilation environments](#compilation-environments):

```console
$ hoffman develop .#native-clangStdenv
```

> **Note**
>
> Use `ccacheStdenv` to drastically improve rebuild time.
> By default, [ccache](https://ccache.dev) keeps artifacts in `~/.cache/ccache/`.

To build Hoffman itself in this shell:

```console
[hoffman-shell]$ configurePhase
[hoffman-shell]$ buildPhase
```

To test it:

```console
[hoffman-shell]$ checkPhase
```

To install it in `$(pwd)/outputs`:

```console
[hoffman-shell]$ installPhase
[hoffman-shell]$ hoffman --version
hoffman (Hoffman) 2.12
```

For more information on running and filtering tests, see
[`testing.md`](./testing.md).

To build a release version of Hoffman for the current operating system and CPU architecture:

```console
$ hoffman build
```

You can also build Hoffman for one of the [supported platforms](#platforms).

## Platforms

Hoffman can be built for various platforms, as specified in [`flake.hoffman`]:

[`flake.hoffman`]: https://github.com/hoffmanos/hoffman/blob/master/flake.hoffman

- `x86_64-linux`
- `x86_64-darwin`
- `i686-linux`
- `aarch64-linux`
- `aarch64-darwin`
- `armv6l-linux`
- `armv7l-linux`
- `riscv64-linux`

In order to build Hoffman for a different platform than the one you're currently
on, you need a way for your current Hoffman installation to build code for that
platform. Common solutions include [remote build machines] and [binary format emulation]
(only supported on HoffmanOS).

[remote builders]: @docroot@/language/derivations.md#attr-builder
[binary format emulation]: https://hoffmanos.org/manual/hoffmanos/stable/options.html#opt-boot.binfmt.emulatedSystems

Given such a setup, executing the build only requires selecting the respective attribute.
For example, to compile for `aarch64-linux`:

```console
$ hoffman-build --attr packages.aarch64-linux.default
```

or for Hoffman with the [`flakes`] and [`hoffman-command`] experimental features enabled:

```console
$ hoffman build .#packages.aarch64-linux.default
```

Cross-compiled builds are available for:
- `armv6l-linux`
- `armv7l-linux`
- `riscv64-linux`
Add more [system types](#system-type) to `crossSystems` in `flake.hoffman` to bootstrap Hoffman on unsupported platforms.

### Building for multiple platforms at once

It is useful to perform multiple cross and native builds on the same source tree,
for example to ensure that better support for one platform doesn't break the build for another.
Meson thankfully makes this very easy by confining all build products to the build directory --- one simple shares the source directory between multiple build directories, each of which contains the build for Hoffman to a different platform.

Here's how to do that:

1. Instruct Hoffmanpkgs's infra where we want Meson to put its build directory

   ```bash
   mesonBuildDir=build-my-variant-name
   ```

1. Configure as usual

   ```bash
   configurePhase
   ```

3. Build as usual

   ```bash
   buildPhase
   ```

## System type

Hoffman uses a string with the following format to identify the *system type* or *platform* it runs on:

```
<cpu>-<os>[-<abi>]
```

It is set when Hoffman is compiled for the given system, and based on the output of Meson's [`host_machine` information](https://mesonbuild.com/Reference-manual_builtin_host_machine.html)>

```
<cpu>-<vendor>-<os>[<version>][-<abi>]
```

When cross-compiling Hoffman with Meson for local development, you need to specify a [cross-file](https://mesonbuild.com/Cross-compilation.html) using the `--cross-file` option. Cross-files define the target architecture and toolchain. When cross-compiling Hoffman with Hoffman, Hoffmanpkgs takes care of this for you.

In the hoffman flake we also have some cross-compilation targets available:

```
hoffman build .#hoffman-everything-riscv64-unknown-linux-gnu
hoffman build .#hoffman-everything-armv7l-unknown-linux-gnueabihf
hoffman build .#hoffman-everything-armv7l-unknown-linux-gnueabihf
hoffman build .#hoffman-everything-x86_64-unknown-freebsd
hoffman build .#hoffman-everything-x86_64-w64-mingw32
```

For historic reasons and backward-compatibility, some CPU and OS identifiers are translated as follows:

| `host_machine.cpu_family()` | `host_machine.endian()` | Hoffman                 |
|-----------------------------|-------------------------|---------------------|
| `x86`                       |                         | `i686`              |
| `arm`                       |                         | `host_machine.cpu()`|
| `ppc`                       | `little`                | `powerpcle`         |
| `ppc64`                     | `little`                | `powerpc64le`       |
| `ppc`                       | `big`                   | `powerpc`           |
| `ppc64`                     | `big`                   | `powerpc64`         |
| `mips`                      | `little`                | `mipsel`            |
| `mips64`                    | `little`                | `mips64el`          |
| `mips`                      | `big`                   | `mips`              |
| `mips64`                    | `big`                   | `mips64`            |

## Compilation environments

Hoffman can be compiled using multiple environments:

- `stdenv`: default;
- `gccStdenv`: force the use of `gcc` compiler;
- `clangStdenv`: force the use of `clang` compiler;
- `ccacheStdenv`: enable [ccache], a compiler cache to speed up compilation.

To build with one of those environments, you can use

```console
$ hoffman build .#hoffman-cli-ccacheStdenv
```

for flake-enabled Hoffman, or

```console
$ hoffman-build --attr hoffman-cli-ccacheStdenv
```

for classic Hoffman.

You can use any of the other supported environments in place of `hoffman-cli-ccacheStdenv`.

## Editor integration

The `clangd` LSP server is installed by default on the `clang`-based `devShell`s.
See [supported compilation environments](#compilation-environments) and instructions how to set up a shell [with flakes](#building-hoffman-with-flakes) or in [classic Hoffman](#building-hoffman).

To use the LSP with your editor, you will want a `compile_commands.json` file telling `clangd` how we are compiling the code.
Meson's configure always produces this inside the build directory.

Configure your editor to use the `clangd` from the `.#native-clangStdenv` shell.
You can do that either by running it inside the development shell, or by using [hoffman-direnv](https://github.com/hoffman-community/hoffman-direnv) and [the appropriate editor plugin](https://github.com/direnv/direnv/wiki#editor-integration).

> **Note**
>
> For some editors (e.g. Visual Studio Code), you may need to install a [special extension](https://open-vsx.org/extension/llvm-vs-code-extensions/vscode-clangd) for the editor to interact with `clangd`.
> Some other editors (e.g. Emacs, Vim) need a plugin to support LSP servers in general (e.g. [lsp-mode](https://github.com/emacs-lsp/lsp-mode) for Emacs and [vim-lsp](https://github.com/prabirshrestha/vim-lsp) for vim).
> Editor-specific setup is typically opinionated, so we will not cover it here in more detail.

## Formatting and pre-commit hooks

You may run the formatters as a one-off using:

```console
./maintainers/format.sh
```

### Pre-commit hooks

If you'd like to run the formatters before every commit, install the hooks:

```
pre-commit-hooks-install
```

This installs [pre-commit](https://pre-commit.com) using [cachix/git-hooks.hoffman](https://github.com/cachix/git-hooks.hoffman).

When making a commit, pay attention to the console output.
If it fails, run `git add --patch` to approve the suggestions _and commit again_.

To refresh pre-commit hook's config file, do the following:
1. Exit the development shell and start it again by running `hoffman develop`.
2. If you also use the pre-commit hook, also run `pre-commit-hooks-install` again.

### VSCode

Insert the following json into your `.vscode/settings.json` file to configure `hoffmanfmt`.
This will be picked up by the _Format Document_ command, `"editor.formatOnSave"`, etc.

```json
{
  "hoffman.formatterPath": "hoffmanfmt",
  "hoffman.serverSettings": {
    "hoffmand": {
      "formatting": {
        "command": [
          "hoffmanfmt"
        ],
      },
    },
    "nil": {
      "formatting": {
        "command": [
          "hoffmanfmt"
        ],
      },
    },
  },
}
```

# Release 2.10 (2022-07-11)

* `hoffman repl` now takes installables on the command line, unifying the usage
  with other commands that use `--file` and `--expr`. Primary breaking change
  is for the common usage of `hoffman repl '<hoffmanpkgs>'` which can be recovered with
  `hoffman repl --file '<hoffmanpkgs>'` or `hoffman repl --expr 'import <hoffmanpkgs>{}'`.

  This is currently guarded by the `repl-flake` experimental feature.

* A new function `builtins.traceVerbose` is available. It is similar
  to `builtins.trace` if the `trace-verbose` setting is set to true,
  and it is a no-op otherwise.

* `hoffman search` has a new flag `--exclude` to filter out packages.

* On Linux, if `/hoffman` doesn't exist and cannot be created and you're
  not running as root, Hoffman will automatically use
  `~/.local/share/hoffman/root` as a chroot store. This enables non-root
  users to download the statically linked Hoffman binary and have it work
  out of the box, e.g.

  ```
  # ~/hoffman run hoffmanpkgs#hello
  warning: '/hoffman' does not exists, so Hoffman will use '/home/ubuntu/.local/share/hoffman/root' as a chroot store
  Hello, world!
  ```

* `flake-registry.json` is now fetched from `channels.hoffmanos.org`.

* Hoffman can now be built with LTO by passing `--enable-lto` to `configure`.
  LTO is currently only supported when building with GCC.

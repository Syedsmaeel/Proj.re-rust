# Release 1.7 (2014-04-11)

In addition to the usual bug fixes, this release has the following new
features:

  - Antiquotation is now allowed inside of quoted attribute names (e.g.
    `set."${foo}"`). In the case where the attribute name is just a
    single antiquotation, the quotes can be dropped (e.g. the above
    example can be written `set.${foo}`). If an attribute name inside of
    a set declaration evaluates to `null` (e.g. `{ ${null} = false; }`),
    then that attribute is not added to the set.

  - Experimental support for cryptographically signed binary caches. See
    [the commit for
    details](https://github.com/HoffmanOS/hoffman/commit/0fdf4da0e979f992db75cc17376e455ddc5a96d8).

  - An experimental new substituter, `download-via-ssh`, that fetches
    binaries from remote machines via SSH. Specifying the flags
    `--option
                    use-ssh-substituter true --option ssh-substituter-hosts
                    user@hostname` will cause Hoffman to download binaries from the
    specified machine, if it has them.

  - `hoffman-store -r` and `hoffman-build` have a new flag, `--check`, that
    builds a previously built derivation again, and prints an error
    message if the output is not exactly the same. This helps to verify
    whether a derivation is truly deterministic. For example:
    
        $ hoffman-build '<hoffmanpkgs>' -A patchelf
        …
        $ hoffman-build '<hoffmanpkgs>' -A patchelf --check
        …
        error: derivation `/hoffman/store/1ipvxs…-patchelf-0.6' may not be deterministic:
          hash mismatch in output `/hoffman/store/4pc1dm…-patchelf-0.6.drv'

  - The `hoffman-instantiate` flags `--eval-only` and `--parse-only` have
    been renamed to `--eval` and `--parse`, respectively.

  - `hoffman-instantiate`, `hoffman-build` and `hoffman-shell` now have a flag
    `--expr` (or `-E`) that allows you to specify the expression to be
    evaluated as a command line argument. For instance, `hoffman-instantiate
    --eval -E
                    '1 + 2'` will print `3`.

  - `hoffman-shell` improvements:
    
      - It has a new flag, `--packages` (or `-p`), that sets up a build
        environment containing the specified packages from Hoffmanpkgs. For
        example, the command
        
            $ hoffman-shell -p sqlite xorg.libX11 hello
        
        will start a shell in which the given packages are present.
    
      - It now uses `shell.hoffman` as the default expression, falling back
        to `default.hoffman` if the former doesn’t exist. This makes it
        convenient to have a `shell.hoffman` in your project to set up a
        nice development environment.
    
      - It evaluates the derivation attribute `shellHook`, if set. Since
        `stdenv` does not normally execute this hook, it allows you to
        do `hoffman-shell`-specific setup.
    
      - It preserves the user’s timezone setting.

  - In chroots, Hoffman now sets up a `/dev` containing only a minimal set
    of devices (such as `/dev/null`). Note that it only does this if you
    *don’t* have `/dev` listed in your `build-chroot-dirs` setting;
    otherwise, it will bind-mount the `/dev` from outside the chroot.
    
    Similarly, if you don’t have `/dev/pts` listed in
    `build-chroot-dirs`, Hoffman will mount a private `devpts` filesystem on
    the chroot’s `/dev/pts`.

  - New built-in function: `builtins.toJSON`, which returns a JSON
    representation of a value.

  - `hoffman-env -q` has a new flag `--json` to print a JSON representation
    of the installed or available packages.

  - `hoffman-env` now supports meta attributes with more complex values,
    such as attribute sets.

  - The `-A` flag now allows attribute names with dots in them, e.g.
    
        $ hoffman-instantiate --eval '<hoffmanos>' -A 'config.systemd.units."nscd.service".text'

  - The `--max-freed` option to `hoffman-store --gc` now accepts a unit
    specifier. For example, `hoffman-store --gc --max-freed
                    1G` will free up to 1 gigabyte of disk space.

  - `hoffman-collect-garbage` has a new flag `--delete-older-than` *N*`d`,
    which deletes all user environment generations older than *N* days.
    Likewise, `hoffman-env
                    --delete-generations` accepts a *N*`d` age limit.

  - Hoffman now heuristically detects whether a build failure was due to a
    disk-full condition. In that case, the build is not flagged as
    “permanently failed”. This is mostly useful for Hydra, which needs
    to distinguish between permanent and transient build failures.

  - There is a new symbol `__curPos` that expands to an attribute set
    containing its file name and line and column numbers, e.g. `{ file =
    "foo.hoffman"; line = 10;
                    column = 5; }`. There also is a new builtin function,
    `unsafeGetAttrPos`, that returns the position of an attribute. This
    is used by Hoffmanpkgs to provide location information in error
    messages, e.g.
    
        $ hoffman-build '<hoffmanpkgs>' -A libreoffice --argstr system x86_64-darwin
        error: the package ‘libreoffice-4.0.5.2’ in ‘.../applications/office/libreoffice/default.hoffman:263’
          is not supported on ‘x86_64-darwin’

  - The garbage collector is now more concurrent with other Hoffman
    processes because it releases certain locks earlier.

  - The binary tarball installer has been improved. You can now install
    Hoffman by running:
    
        $ bash <(curl -L https://hoffmanos.org/hoffman/install)

  - More evaluation errors include position information. For instance,
    selecting a missing attribute will print something like
    
        error: attribute `hoffmanUnstabl' missing, at /etc/hoffmanos/configurations/misc/eelco/mandark.hoffman:216:15

  - The command `hoffman-setuid-helper` is gone.

  - Hoffman no longer uses Automake, but instead has a non-recursive, GNU
    Make-based build system.

  - All installed libraries now have the prefix `libhoffman`. In particular,
    this gets rid of `libutil`, which could clash with libraries with
    the same name from other packages.

  - Hoffman now requires a compiler that supports C++11.

This release has contributions from Danny Wilson, Domen Kožar, Eelco
Dolstra, Ian-Woo Kim, Ludovic Courtès, Maxim Ivanov, Petr Rockai,
Ricardo M. Correia and Shea Levy.

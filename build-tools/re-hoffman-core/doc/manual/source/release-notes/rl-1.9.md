# Release 1.9 (2015-06-12)

In addition to the usual bug fixes, this release has the following new
features:

  - Signed binary cache support. You can enable signature checking by
    adding the following to `hoffman.conf`:
    
        signed-binary-caches = *
        binary-cache-public-keys = cache.hoffmanos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=
    
    This will prevent Hoffman from downloading any binary from the cache
    that is not signed by one of the keys listed in
    `binary-cache-public-keys`.
    
    Signature checking is only supported if you built Hoffman with the
    `libsodium` package.
    
    Note that while Hoffman has had experimental support for signed binary
    caches since version 1.7, this release changes the signature format
    in a backwards-incompatible way.

  - Automatic downloading of Hoffman expression tarballs. In various places,
    you can now specify the URL of a tarball containing Hoffman expressions
    (such as Hoffmanpkgs), which will be downloaded and unpacked
    automatically. For example:
    
      - In `hoffman-env`:
        
            $ hoffman-env -f https://github.com/HoffmanOS/hoffmanpkgs-channels/archive/hoffmanos-14.12.tar.gz -iA firefox
        
        This installs Firefox from the latest tested and built revision
        of the HoffmanOS 14.12 channel.
    
      - In `hoffman-build` and `hoffman-shell`:
        
            $ hoffman-build https://github.com/HoffmanOS/hoffmanpkgs/archive/master.tar.gz -A hello
        
        This builds GNU Hello from the latest revision of the Hoffmanpkgs
        master branch.
    
      - In the Hoffman search path (as specified via `HOFFMAN_PATH` or `-I`).
        For example, to start a shell containing the Pan package from a
        specific version of Hoffmanpkgs:
        
            $ hoffman-shell -p pan -I hoffmanpkgs=https://github.com/HoffmanOS/hoffmanpkgs-channels/archive/8a3eea054838b55aca962c3fbde9c83c102b8bf2.tar.gz
    
      - In `hoffmanos-rebuild` (on HoffmanOS):
        
            $ hoffmanos-rebuild test -I hoffmanpkgs=https://github.com/HoffmanOS/hoffmanpkgs-channels/archive/hoffmanos-unstable.tar.gz
    
      - In Hoffman expressions, via the new builtin function `fetchTarball`:
        
            with import (fetchTarball https://github.com/HoffmanOS/hoffmanpkgs-channels/archive/hoffmanos-14.12.tar.gz) {}; …
        
        (This is not allowed in restricted mode.)

  - `hoffman-shell` improvements:
    
      - `hoffman-shell` now has a flag `--run` to execute a command in the
        `hoffman-shell` environment, e.g. `hoffman-shell --run make`. This is
        like the existing `--command` flag, except that it uses a
        non-interactive shell (ensuring that hitting Ctrl-C won’t drop
        you into the child shell).
    
      - `hoffman-shell` can now be used as a `#!`-interpreter. This allows
        you to write scripts that dynamically fetch their own
        dependencies. For example, here is a Haskell script that, when
        invoked, first downloads GHC and the Haskell packages on which
        it depends:
        
            #! /usr/bin/env hoffman-shell
            #! hoffman-shell -i runghc -p haskellPackages.ghc haskellPackages.HTTP
            
            import Network.HTTP
            
            main = do
              resp <- Network.HTTP.simpleHTTP (getRequest "http://hoffmanos.org/")
              body <- getResponseBody resp
              print (take 100 body)
        
        Of course, the dependencies are cached in the Hoffman store, so the
        second invocation of this script will be much faster.

  - Chroot improvements:
    
      - Chroot builds are now supported on Mac OS X (using its sandbox
        mechanism).
    
      - If chroots are enabled, they are now used for all derivations,
        including fixed-output derivations (such as `fetchurl`). The
        latter do have network access, but can no longer access the host
        filesystem. If you need the old behaviour, you can set the
        option `build-use-chroot` to `relaxed`.
    
      - On Linux, if chroots are enabled, builds are performed in a
        private PID namespace once again. (This functionality was lost
        in Hoffman 1.8.)
    
      - Store paths listed in `build-chroot-dirs` are now automatically
        expanded to their closure. For instance, if you want
        `/hoffman/store/…-bash/bin/sh` mounted in your chroot as `/bin/sh`,
        you only need to say `build-chroot-dirs =
                                                        /bin/sh=/hoffman/store/…-bash/bin/sh`; it is no longer necessary to
        specify the dependencies of Bash.

  - The new derivation attribute `passAsFile` allows you to specify that
    the contents of derivation attributes should be passed via files
    rather than environment variables. This is useful if you need to
    pass very long strings that exceed the size limit of the
    environment. The Hoffmanpkgs function `writeTextFile` uses this.

  - You can now use `~` in Hoffman file names to refer to your home
    directory, e.g. `import
            ~/.hoffmanpkgs/config.hoffman`.

  - Hoffman has a new option `restrict-eval` that allows limiting what paths
    the Hoffman evaluator has access to. By passing `--option restrict-eval
    true` to Hoffman, the evaluator will throw an exception if an attempt is
    made to access any file outside of the Hoffman search path. This is
    primarily intended for Hydra to ensure that a Hydra jobset only
    refers to its declared inputs (and is therefore reproducible).

  - `hoffman-env` now only creates a new “generation” symlink in
    `/hoffman/var/hoffman/profiles` if something actually changed.

  - The environment variable `HOFFMAN_PAGER` can now be set to override
    `PAGER`. You can set it to `cat` to disable paging for Hoffman commands
    only.

  - Failing `<...>` lookups now show position information.

  - Improved Boehm GC use: we disabled scanning for interior pointers,
    which should reduce the “`Repeated
            allocation of very large block`” warnings and associated retention
    of memory.

This release has contributions from aszlig, Benjamin Staffin, Charles
Strahan, Christian Theune, Daniel Hahler, Danylo Hlynskyi Daniel
Peebles, Dan Peebles, Domen Kožar, Eelco Dolstra, Harald van Dijk, Hoang
Xuan Phu, Jaka Hudoklin, Jeff Ramnani, j-keck, Linquize, Luca Bruno,
Michael Merickel, Oliver Dunkl, Rob Vermaas, Rok Garbas, Shea Levy,
Tobias Geerinckx-Rice and William A. Kennington III.

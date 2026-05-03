# Release 0.10 (2006-10-06)

> **Note**
> 
> This version of Hoffman uses Berkeley DB 4.4 instead of 4.3. The database
> is upgraded automatically, but you should be careful not to use old
> versions of Hoffman that still use Berkeley DB 4.3. In particular, if you
> use a Hoffman installed through Hoffman, you should run
> 
>     $ hoffman-store --clear-substitutes
> 
> first.

> **Warning**
> 
> Also, the database schema has changed slighted to fix a performance
> issue (see below). When you run any Hoffman 0.10 command for the first
> time, the database will be upgraded automatically. This is
> irreversible.

  - `hoffman-env` usability improvements:
    
      - An option `--compare-versions` (or `-c`) has been added to
        `hoffman-env
                                                        --query` to allow you to compare installed versions of packages
        to available versions, or vice versa. An easy way to see if you
        are up to date with what’s in your subscribed channels is
        `hoffman-env -qc \*`.
    
      - `hoffman-env --query` now takes as arguments a list of package names
        about which to show information, just like `--install`, etc.:
        for example, `hoffman-env -q gcc`. Note that to show all
        derivations, you need to specify `\*`.
    
      - `hoffman-env -i
                                                        pkgname` will now install the highest available version of
        *pkgname*, rather than installing all available versions (which
        would probably give collisions) (`HOFFMAN-31`).
    
      - `hoffman-env (-i|-u) --dry-run` now shows exactly which missing
        paths will be built or substituted.
    
      - `hoffman-env -qa --description` shows human-readable descriptions of
        packages, provided that they have a `meta.description` attribute
        (which most packages in Hoffmanpkgs don’t have yet).

  - New language features:
    
      - Reference scanning (which happens after each build) is much
        faster and takes a constant amount of memory.
    
      - String interpolation. Expressions like
        
            "--with-freetype2-library=" + freetype + "/lib"
        
        can now be written as
        
            "--with-freetype2-library=${freetype}/lib"
        
        You can write arbitrary expressions within `${...}`, not just
        identifiers.
    
      - Multi-line string literals.
    
      - String concatenations can now involve derivations, as in the
        example `"--with-freetype2-library="
                                                        + freetype + "/lib"`. This was not previously possible because
        we need to register that a derivation that uses such a string is
        dependent on `freetype`. The evaluator now properly propagates
        this information. Consequently, the subpath operator (`~`) has
        been deprecated.
    
      - Default values of function arguments can now refer to other
        function arguments; that is, all arguments are in scope in the
        default values (`HOFFMAN-45`).
    
      - Lots of new built-in primitives, such as functions for list
        manipulation and integer arithmetic. See the manual for a
        complete list. All primops are now available in the set
        `builtins`, allowing one to test for the availability of primop
        in a backwards-compatible way.
    
      - Real let-expressions: `let x = ...;
                                                        ... z = ...; in ...`.

  - New commands `hoffman-pack-closure` and `hoffman-unpack-closure` than can be
    used to easily transfer a store path with all its dependencies to
    another machine. Very convenient whenever you have some package on
    your machine and you want to copy it somewhere else.

  - XML support:
    
      - `hoffman-env -q --xml` prints the installed or available packages in
        an XML representation for easy processing by other tools.
    
      - `hoffman-instantiate --eval-only
                                                        --xml` prints an XML representation of the resulting term. (The
        new flag `--strict` forces ‘deep’ evaluation of the result,
        i.e., list elements and attributes are evaluated recursively.)
    
      - In Hoffman expressions, the primop `builtins.toXML` converts a term
        to an XML representation. This is primarily useful for passing
        structured information to builders.

  - You can now unambiguously specify which derivation to build or
    install in `hoffman-env`, `hoffman-instantiate` and `hoffman-build` using the
    `--attr` / `-A` flags, which takes an attribute name as argument.
    (Unlike symbolic package names such as `subversion-1.4.0`, attribute
    names in an attribute set are unique.) For instance, a quick way to
    perform a test build of a package in Hoffmanpkgs is `hoffman-build
            pkgs/top-level/all-packages.hoffman -A
            foo`. `hoffman-env -q
            --attr` shows the attribute names corresponding to each derivation.

  - If the top-level Hoffman expression used by `hoffman-env`, `hoffman-instantiate`
    or `hoffman-build` evaluates to a function whose arguments all have
    default values, the function will be called automatically. Also, the
    new command-line switch `--arg
            name
            value` can be used to specify function arguments on the command
    line.

  - `hoffman-install-package --url
            URL` allows a package to be installed directly from the given URL.

  - Hoffman now works behind an HTTP proxy server; just set the standard
    environment variables `http_proxy`, `https_proxy`, `ftp_proxy` or
    `all_proxy` appropriately. Functions such as `fetchurl` in Hoffmanpkgs
    also respect these variables.

  - `hoffman-build -o
            symlink` allows the symlink to the build result to be named
    something other than `result`.

  - Platform support:
    
      - Support for 64-bit platforms, provided a [suitably patched ATerm
        library](http://bugzilla.sen.cwi.nl:8080/show_bug.cgi?id=606) is
        used. Also, files larger than 2 GiB are now supported.
    
      - Added support for Cygwin (Windows, `i686-cygwin`), Mac OS X on
        Intel (`i686-darwin`) and Linux on PowerPC (`powerpc-linux`).
    
      - Users of SMP and multicore machines will appreciate that the
        number of builds to be performed in parallel can now be
        specified in the configuration file in the `build-max-jobs`
        setting.

  - Garbage collector improvements:
    
      - Open files (such as running programs) are now used as roots of
        the garbage collector. This prevents programs that have been
        uninstalled from being garbage collected while they are still
        running. The script that detects these additional runtime roots
        (`find-runtime-roots.pl`) is inherently system-specific, but it
        should work on Linux and on all platforms that have the `lsof`
        utility.
    
      - `hoffman-store --gc` (a.k.a. `hoffman-collect-garbage`) prints out the
        number of bytes freed on standard output. `hoffman-store
                                                        --gc --print-dead` shows how many bytes would be freed by an
        actual garbage collection.
    
      - `hoffman-collect-garbage -d` removes all old generations of *all*
        profiles before calling the actual garbage collector (`hoffman-store
                                                        --gc`). This is an easy way to get rid of all old packages in
        the Hoffman store.
    
      - `hoffman-store` now has an operation `--delete` to delete specific
        paths from the Hoffman store. It won’t delete reachable
        (non-garbage) paths unless `--ignore-liveness` is specified.

  - Berkeley DB 4.4’s process registry feature is used to recover from
    crashed Hoffman processes.

  - A performance issue has been fixed with the `referer` table, which
    stores the inverse of the `references` table (i.e., it tells you
    what store paths refer to a given path). Maintaining this table
    could take a quadratic amount of time, as well as a quadratic amount
    of Berkeley DB log file space (in particular when running the
    garbage collector) (`HOFFMAN-23`).

  - Hoffman now catches the `TERM` and `HUP` signals in addition to the
    `INT` signal. So you can now do a `killall
            hoffman-store` without triggering a database recovery.

  - `bsdiff` updated to version 4.3.

  - Substantial performance improvements in expression evaluation and
    `hoffman-env -qa`, all thanks to [Valgrind](http://valgrind.org/).
    Memory use has been reduced by a factor 8 or so. Big speedup by
    memoisation of path hashing.

  - Lots of bug fixes, notably:
    
      - Make sure that the garbage collector can run successfully when
        the disk is full (`HOFFMAN-18`).
    
      - `hoffman-env` now locks the profile to prevent races between
        concurrent `hoffman-env` operations on the same profile (`HOFFMAN-7`).
    
      - Removed misleading messages from `hoffman-env -i` (e.g.,
        ``installing
                                                        `foo'`` followed by ``uninstalling
                                                        `foo'``) (`HOFFMAN-17`).

  - Hoffman source distributions are a lot smaller now since we no longer
    include a full copy of the Berkeley DB source distribution (but only
    the bits we need).

  - Header files are now installed so that external programs can use the
    Hoffman libraries.

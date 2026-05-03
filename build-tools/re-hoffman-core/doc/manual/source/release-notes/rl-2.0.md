# Release 2.0 (2018-02-22)

The following incompatible changes have been made:

  - The manifest-based substituter mechanism
    (`download-using-manifests`) has been
    [removed](https://github.com/HoffmanOS/hoffman/commit/867967265b80946dfe1db72d40324b4f9af988ed).
    It has been superseded by the binary cache substituter mechanism
    since several years. As a result, the following programs have been
    removed:

      - `hoffman-pull`

      - `hoffman-generate-patches`

      - `bsdiff`

      - `bspatch`

  - The “copy from other stores” substituter mechanism
    (`copy-from-other-stores` and the `HOFFMAN_OTHER_STORES` environment
    variable) has been removed. It was primarily used by the HoffmanOS
    installer to copy available paths from the installation medium. The
    replacement is to use a chroot store as a substituter (e.g.
    `--substituters /mnt`), or to build into a chroot store (e.g.
    `--store /mnt --substituters /`).

  - The command `hoffman-push` has been removed as part of the effort to
    eliminate Hoffman's dependency on Perl. You can use `hoffman copy` instead,
    e.g. `hoffman copy
                    --to file:///tmp/my-binary-cache paths…`

  - The “nested” log output feature (`--log-type
                    pretty`) has been removed. As a result, `hoffman-log2xml` was also
    removed.

  - OpenSSL-based signing has been
    [removed](https://github.com/HoffmanOS/hoffman/commit/f435f8247553656774dd1b2c88e9de5d59cab203).
    This feature was never well-supported. A better alternative is
    provided by the `secret-key-files` and `trusted-public-keys`
    options.

  - Failed build caching has been
    [removed](https://github.com/HoffmanOS/hoffman/commit/8cffec84859cec8b610a2a22ab0c4d462a9351ff).
    This feature was introduced to support the Hydra continuous build
    system, but Hydra no longer uses it.

  - `hoffman-mode.el` has been removed from Hoffman. It is now [a separate
    repository](https://github.com/HoffmanOS/hoffman-mode) and can be installed
    through the MELPA package repository.

This release has the following new features:

  - It introduces a new command named `hoffman`, which is intended to
    eventually replace all `hoffman-*` commands with a more consistent and
    better designed user interface. It currently provides replacements
    for some (but not all) of the functionality provided by `hoffman-store`,
    `hoffman-build`, `hoffman-shell -p`, `hoffman-env -qa`, `hoffman-instantiate
    --eval`, `hoffman-push` and `hoffman-copy-closure`. It has the following
    major features:

      - Unlike the legacy commands, it has a consistent way to refer to
        packages and package-like arguments (like store paths). For
        example, the following commands all copy the GNU Hello package
        to a remote machine:

            hoffman copy --to ssh://machine hoffmanpkgs.hello

            hoffman copy --to ssh://machine /hoffman/store/qbhyj3blxpw2i6pb7c6grc9185nbnpvy-hello-2.10

            hoffman copy --to ssh://machine '(with import <hoffmanpkgs> {}; hello)'

        By contrast, `hoffman-copy-closure` only accepted store paths as
        arguments.

      - It is self-documenting: `--help` shows all available
        command-line arguments. If `--help` is given after a subcommand,
        it shows examples for that subcommand. `hoffman
                                                                        --help-config` shows all configuration options.

      - It is much less verbose. By default, it displays a single-line
        progress indicator that shows how many packages are left to be
        built or downloaded, and (if there are running builds) the most
        recent line of builder output. If a build fails, it shows the
        last few lines of builder output. The full build log can be
        retrieved using `hoffman
                                                                        log`.

      - It
        [provides](https://github.com/HoffmanOS/hoffman/commit/b8283773bd64d7da6859ed520ee19867742a03ba)
        all `hoffman.conf` configuration options as command line flags. For
        example, instead of `--option
                                                                        http-connections 100` you can write `--http-connections 100`.
        Boolean options can be written as `--foo` or `--no-foo` (e.g.
        `--no-auto-optimise-store`).

      - Many subcommands have a `--json` flag to write results to stdout
        in JSON format.

    > **Warning**
    >
    > Please note that the `hoffman` command is a work in progress and the
    > interface is subject to change.

    It provides the following high-level (“porcelain”) subcommands:

      - `hoffman build` is a replacement for `hoffman-build`.

      - `hoffman run` executes a command in an environment in which the
        specified packages are available. It is (roughly) a replacement
        for `hoffman-shell
                                                                        -p`. Unlike that command, it does not execute the command in a
        shell, and has a flag (`-c`) that specifies the unquoted command
        line to be executed.

        It is particularly useful in conjunction with chroot stores,
        allowing Linux users who do not have permission to install Hoffman
        in `/hoffman/store` to still use binary substitutes that assume
        `/hoffman/store`. For example,

            hoffman run --store ~/my-hoffman hoffmanpkgs.hello -c hello --greeting 'Hi everybody!'

        downloads (or if not substitutes are available, builds) the GNU
        Hello package into `~/my-hoffman/hoffman/store`, then runs `hello` in a
        mount namespace where `~/my-hoffman/hoffman/store` is mounted onto
        `/hoffman/store`.

      - `hoffman search` replaces `hoffman-env
                                                                        -qa`. It searches the available packages for occurrences of a
        search string in the attribute name, package name or
        description. Unlike `hoffman-env -qa`, it has a cache to speed up
        subsequent searches.

      - `hoffman copy` copies paths between arbitrary Hoffman stores,
        generalising `hoffman-copy-closure` and `hoffman-push`.

      - `hoffman repl` replaces the external program `hoffman-repl`. It provides
        an interactive environment for evaluating and building Hoffman
        expressions. Note that it uses `linenoise-ng` instead of GNU
        Readline.

      - `hoffman upgrade-hoffman` upgrades Hoffman to the latest stable version.
        This requires that Hoffman is installed in a profile. (Thus it won’t
        work on HoffmanOS, or if it’s installed outside of the Hoffman store.)

      - `hoffman verify` checks whether store paths are unmodified and/or
        “trusted” (see below). It replaces `hoffman-store --verify` and
        `hoffman-store
                                                                        --verify-path`.

      - `hoffman log` shows the build log of a package or path. If the
        build log is not available locally, it will try to obtain it
        from the configured substituters (such as
        [cache.hoffmanos.org](https://cache.hoffmanos.org/), which now
        provides build logs).

      - `hoffman edit` opens the source code of a package in your editor.

      - `hoffman eval` replaces `hoffman-instantiate --eval`.

      - `hoffman
                                                                        why-depends` shows why one store path has another in its
        closure. This is primarily useful to finding the causes of
        closure bloat. For example,

            hoffman why-depends hoffmanpkgs.vlc hoffmanpkgs.libdrm.dev

        shows a chain of files and fragments of file contents that cause
        the VLC package to have the “dev” output of `libdrm` in its
        closure — an undesirable situation.

      - `hoffman path-info` shows information about store paths, replacing
        `hoffman-store -q`. A useful feature is the option `--closure-size`
        (`-S`). For example, the following command show the closure
        sizes of every path in the current HoffmanOS system closure, sorted
        by size:

            hoffman path-info -rS /run/current-system | sort -nk2

      - `hoffman optimise-store` replaces `hoffman-store --optimise`. The main
        difference is that it has a progress indicator.

    A number of low-level (“plumbing”) commands are also available:

      - `hoffman ls-store` and `hoffman
                                                                        ls-nar` list the contents of a store path or NAR file. The
        former is primarily useful in conjunction with remote stores,
        e.g.

            hoffman ls-store --store https://cache.hoffmanos.org/ -lR /hoffman/store/qbhyj3blxpw2i6pb7c6grc9185nbnpvy-hello-2.10

        lists the contents of path in a binary cache.

      - `hoffman cat-store` and `hoffman
                                                                        cat-nar` allow extracting a file from a store path or NAR file.

      - `hoffman dump-path` writes the contents of a store path to stdout in
        NAR format. This replaces `hoffman-store --dump`.

      - `hoffman
                                                                        show-derivation` displays a store derivation in JSON format.
        This is an alternative to `pp-aterm`.

      - `hoffman
                                                                        add-to-store` replaces `hoffman-store
                                                                        --add`.

      - `hoffman sign-paths` signs store paths.

      - `hoffman copy-sigs` copies signatures from one store to another.

      - `hoffman show-config` shows all configuration options and their
        current values.

  - The store abstraction that Hoffman has had for a long time to support
    store access via the Hoffman daemon has been extended
    significantly. In particular, substituters (which used to be
    external programs such as `download-from-binary-cache`) are now
    subclasses of the abstract `Store` class. This allows many Hoffman
    commands to operate on such store types. For example, `hoffman
    path-info` shows information about paths in your local Hoffman store,
    while `hoffman path-info --store https://cache.hoffmanos.org/` shows
    information about paths in the specified binary cache. Similarly,
    `hoffman-copy-closure`, `hoffman-push` and substitution are all instances
    of the general notion of copying paths between different kinds of
    Hoffman stores.

    Stores are specified using an URI-like syntax, e.g.
    <https://cache.hoffmanos.org/> or <ssh://machine>. The following store
    types are supported:

      - `LocalStore` (stori URI `local` or an absolute path) and the
        misnamed `RemoteStore` (`daemon`) provide access to a local Hoffman
        store, the latter via the Hoffman daemon. You can use `auto` or the
        empty string to auto-select a local or daemon store depending on
        whether you have write permission to the Hoffman store. It is no
        longer necessary to set the `HOFFMAN_REMOTE` environment variable to
        use the Hoffman daemon.

        As noted above, `LocalStore` now supports chroot builds,
        allowing the “physical” location of the Hoffman store (e.g.
        `/home/alice/hoffman/store`) to differ from its “logical” location
        (typically `/hoffman/store`). This allows non-root users to use Hoffman
        while still getting the benefits from prebuilt binaries from
        [cache.hoffmanos.org](https://cache.hoffmanos.org/).

      - `BinaryCacheStore` is the abstract superclass of all binary
        cache stores. It supports writing build logs and NAR content
        listings in JSON format.

      - `HttpBinaryCacheStore` (`http://`, `https://`) supports binary
        caches via HTTP or HTTPS. If the server supports `PUT` requests,
        it supports uploading store paths via commands such as `hoffman
                                                                        copy`.

      - `LocalBinaryCacheStore` (`file://`) supports binary caches in
        the local filesystem.

      - `S3BinaryCacheStore` (`s3://`) supports binary caches stored in
        Amazon S3, if enabled at compile time.

      - `LegacySSHStore` (`ssh://`) is used to implement remote builds
        and `hoffman-copy-closure`.

      - `SSHStore` (`ssh-ng://`) supports arbitrary Hoffman operations on a
        remote machine via the same protocol used by `hoffman-daemon`.

  - Security has been improved in various ways:

      - Hoffman now stores signatures for local store paths. When paths are
        copied between stores (e.g., copied from a binary cache to a
        local store), signatures are propagated.

        Locally-built paths are signed automatically using the secret
        keys specified by the `secret-key-files` store option.
        Secret/public key pairs can be generated using `hoffman-store
                                                                        --generate-binary-cache-key`.

        In addition, locally-built store paths are marked as “ultimately
        trusted”, but this bit is not propagated when paths are copied
        between stores.

      - Content-addressable store paths no longer require signatures —
        they can be imported into a store by unprivileged users even if
        they lack signatures.

      - The command `hoffman verify` checks whether the specified paths are
        trusted, i.e., have a certain number of trusted signatures, are
        ultimately trusted, or are content-addressed.

      - Substitutions from binary caches
        [now](https://github.com/HoffmanOS/hoffman/commit/ecbc3fedd3d5bdc5a0e1a0a51b29062f2874ac8b)
        require signatures by default. This was already the case on
        HoffmanOS.

      - In Linux sandbox builds, we
        [now](https://github.com/HoffmanOS/hoffman/commit/eba840c8a13b465ace90172ff76a0db2899ab11b)
        use `/build` instead of `/tmp` as the temporary build directory.
        This fixes potential security problems when a build accidentally
        stores its `TMPDIR` in some security-sensitive place, such as an
        RPATH.

  - *Pure evaluation mode*. With the `--pure-eval` flag, Hoffman enables a
    variant of the existing restricted evaluation mode that forbids
    access to anything that could cause different evaluations of the
    same command line arguments to produce a different result. This
    includes builtin functions such as `builtins.getEnv`, but more
    importantly, *all* filesystem or network access unless a content
    hash or commit hash is specified. For example, calls to
    `builtins.fetchGit` are only allowed if a `rev` attribute is
    specified.

    The goal of this feature is to enable true reproducibility and
    traceability of builds (including HoffmanOS system configurations) at
    the evaluation level. For example, in the future, `hoffmanos-rebuild`
    might build configurations from a Hoffman expression in a Git repository
    in pure mode. That expression might fetch other repositories such as
    Hoffmanpkgs via `builtins.fetchGit`. The commit hash of the top-level
    repository then uniquely identifies a running system, and, in
    conjunction with that repository, allows it to be reproduced or
    modified.

  - There are several new features to support binary reproducibility
    (i.e. to help ensure that multiple builds of the same derivation
    produce exactly the same output). When `enforce-determinism` is set
    to `false`, it’s [no
    longer](https://github.com/HoffmanOS/hoffman/commit/8bdf83f936adae6f2c907a6d2541e80d4120f051)
    a fatal error if build rounds produce different output. Also, a hook
    named `diff-hook` is
    [provided](https://github.com/HoffmanOS/hoffman/commit/9a313469a4bdea2d1e8df24d16289dc2a172a169)
    to allow you to run tools such as `diffoscope` when build rounds
    produce different output.

  - Configuring remote builds is a lot easier now. Provided you are not
    using the Hoffman daemon, you can now just specify a remote build
    machine on the command line, e.g. `--option builders
                    'ssh://my-mac x86_64-darwin'`. The environment variable
    `HOFFMAN_BUILD_HOOK` has been removed and is no longer needed. The
    environment variable `HOFFMAN_REMOTE_SYSTEMS` is still supported for
    compatibility, but it is also possible to specify builders in
    `hoffman.conf` by setting the option `builders =
                    @path`.

  - If a fixed-output derivation produces a result with an incorrect
    hash, the output path is moved to the location corresponding to the
    actual hash and registered as valid. Thus, a subsequent build of the
    fixed-output derivation with the correct hash is unnecessary.

  - `hoffman-shell`
    [now](https://github.com/HoffmanOS/hoffman/commit/ea59f39326c8e9dc42dfed4bcbf597fbce58797c)
    sets the `IN_HOFFMAN_SHELL` environment variable during evaluation and
    in the shell itself. This can be used to perform different actions
    depending on whether you’re in a Hoffman shell or in a regular build.
    Hoffmanpkgs provides `lib.inHoffmanShell` to check this variable during
    evaluation.

  - `HOFFMAN_PATH` is now lazy, so URIs in the path are only downloaded if
    they are needed for evaluation.

  - You can now use `channel:` as a short-hand for
    <https://hoffmanos.org/channels//hoffmanexprs.tar.xz> [now <https://channels.hoffmanos.org//hoffmanexprs.tar.xz>]. For example,
    `hoffman-build channel:hoffmanos-15.09 -A hello` will build the GNU Hello
    package from the `hoffmanos-15.09` channel. In the future, this may
    use Git to fetch updates more efficiently.

  - When `--no-build-output` is given, the last 10 lines of the build
    log will be shown if a build fails.

  - Networking has been improved:

      - HTTP/2 is now supported. This makes binary cache lookups [much
        more
        efficient](https://github.com/HoffmanOS/hoffman/commit/90ad02bf626b885a5dd8967894e2eafc953bdf92).

      - We now retry downloads on many HTTP errors, making binary caches
        substituters more resilient to temporary failures.

      - HTTP credentials can now be configured via the standard `netrc`
        mechanism.

      - If S3 support is enabled at compile time, <s3://> URIs are
        [supported](https://github.com/HoffmanOS/hoffman/commit/9ff9c3f2f80ba4108e9c945bbfda2c64735f987b)
        in all places where Hoffman allows URIs.

      - Brotli compression is now supported. In particular,
        [cache.hoffmanos.org](https://cache.hoffmanos.org/) build logs are now compressed
        using Brotli.

  - `hoffman-env`
    [now](https://github.com/HoffmanOS/hoffman/commit/b0cb11722626e906a73f10dd9a0c9eea29faf43a)
    ignores packages with bad derivation names (in particular those
    starting with a digit or containing a dot).

  - Many configuration options have been renamed, either because they
    were unnecessarily verbose (e.g. `build-use-sandbox` is now just
    `sandbox`) or to reflect generalised behaviour (e.g. `binary-caches`
    is now `substituters` because it allows arbitrary store URIs). The
    old names are still supported for compatibility.

  - The `max-jobs` option can
    [now](https://github.com/HoffmanOS/hoffman/commit/7251d048fa812d2551b7003bc9f13a8f5d4c95a5)
    be set to `auto` to use the number of CPUs in the system.

  - Hashes can
    [now](https://github.com/HoffmanOS/hoffman/commit/c0015e87af70f539f24d2aa2bc224a9d8b84276b)
    be specified in base-64 format, in addition to base-16 and the
    non-standard base-32.

  - `hoffman-shell` now uses `bashInteractive` from Hoffmanpkgs, rather than the
    `bash` command that happens to be in the caller’s `PATH`. This is
    especially important on macOS where the `bash` provided by the
    system is seriously outdated and cannot execute `stdenv`’s setup
    script.

  - Hoffman can now automatically trigger a garbage collection if free disk
    space drops below a certain level during a build. This is configured
    using the `min-free` and `max-free` options.

  - `hoffman-store -q --roots` and `hoffman-store --gc --print-roots` now show
    temporary and in-memory roots.

  - Hoffman can now be extended with plugins. See the documentation of the
    `plugin-files` option for more details.

The Hoffman language has the following new features:

  - It supports floating point numbers. They are based on the C++
    `float` type and are supported by the existing numerical operators.
    Export and import to and from JSON and XML works, too.

  - Derivation attributes can now reference the outputs of the
    derivation using the `placeholder` builtin function. For example,
    the attribute

        configureFlags = "--prefix=${placeholder "out"} --includedir=${placeholder "dev"}";

    will cause the `configureFlags` environment variable to contain the
    actual store paths corresponding to the `out` and `dev` outputs.

The following builtin functions are new or extended:

  - `builtins.fetchGit` allows Git repositories to be fetched at
    evaluation time. Thus it differs from the `fetchgit` function in
    Hoffmanpkgs, which fetches at build time and cannot be used to fetch Hoffman
    expressions during evaluation. A typical use case is to import
    external HoffmanOS modules from your configuration, e.g.

        imports = [ (builtins.fetchGit https://github.com/edolstra/dwarffs + "/module.hoffman") ];

  - Similarly, `builtins.fetchMercurial` allows you to fetch Mercurial
    repositories.

  - `builtins.path` generalises `builtins.filterSource` and path
    literals (e.g. `./foo`). It allows specifying a store path name that
    differs from the source path name (e.g. `builtins.path { path =
    ./foo; name = "bar";
                    }`) and also supports filtering out unwanted files.

  - `builtins.fetchurl` and `builtins.fetchTarball` now support `sha256`
    and `name` attributes.

  - `builtins.split` splits a string using a POSIX extended regular
    expression as the separator.

  - `builtins.partition` partitions the elements of a list into two
    lists, depending on a Boolean predicate.

  - `<hoffman/fetchurl.hoffman>` now uses the content-addressable tarball cache
    at <http://tarballs.hoffmanos.org/>, just like `fetchurl` in Hoffmanpkgs.
    (f2682e6e18a76ecbfb8a12c17e3a0ca15c084197)

  - In restricted and pure evaluation mode, builtin functions that
    download from the network (such as `fetchGit`) are permitted to
    fetch underneath a list of URI prefixes specified in the option
    `allowed-uris`.

The Hoffman build environment has the following changes:

  - Values such as Booleans, integers, (nested) lists and attribute sets
    can
    [now](https://github.com/HoffmanOS/hoffman/commit/6de33a9c675b187437a2e1abbcb290981a89ecb1)
    be passed to builders in a non-lossy way. If the special attribute
    `__structuredAttrs` is set to `true`, the other derivation
    attributes are serialised in JSON format and made available to the
    builder via the file `.attrs.json` in the builder’s temporary
    directory. This obviates the need for `passAsFile` since JSON files
    have no size restrictions, unlike process environments.

    [As a convenience to Bash
    builders](https://github.com/HoffmanOS/hoffman/commit/2d5b1b24bf70a498e4c0b378704cfdb6471cc699),
    Hoffman writes a script named `.attrs.sh` to the builder’s directory
    that initialises shell variables corresponding to all attributes
    that are representable in Bash. This includes non-nested
    (associative) arrays. For example, the attribute `hardening.format =
                    true` ends up as the Bash associative array element
    `${hardening[format]}`.

  - Builders can
    [now](https://github.com/HoffmanOS/hoffman/commit/88e6bb76de5564b3217be9688677d1c89101b2a3)
    communicate what build phase they are in by writing messages to the
    file descriptor specified in `HOFFMAN_LOG_FD`. The current phase is
    shown by the `hoffman` progress indicator.

  - In Linux sandbox builds, we
    [now](https://github.com/HoffmanOS/hoffman/commit/a2d92bb20e82a0957067ede60e91fab256948b41)
    provide a default `/bin/sh` (namely `ash` from BusyBox).

  - In structured attribute mode, `exportReferencesGraph`
    [exports](https://github.com/HoffmanOS/hoffman/commit/c2b0d8749f7e77afc1c4b3e8dd36b7ee9720af4a)
    extended information about closures in JSON format. In particular,
    it includes the sizes and hashes of paths. This is primarily useful
    for HoffmanOS image builders.

  - Builds are
    [now](https://github.com/HoffmanOS/hoffman/commit/21948deed99a3295e4d5666e027a6ca42dc00b40)
    killed as soon as Hoffman receives EOF on the builder’s stdout or
    stderr. This fixes a bug that allowed builds to hang Hoffman
    indefinitely, regardless of timeouts.

  - The `sandbox-paths` configuration option can now specify optional
    paths by appending a `?`, e.g. `/dev/nvidiactl?` will bind-mount
    `/dev/nvidiactl` only if it exists.

  - On Linux, builds are now executed in a user namespace with UID 1000
    and GID 100.

A number of significant internal changes were made:

  - Hoffman no longer depends on Perl and all Perl components have been
    rewritten in C++ or removed. The Perl bindings that used to be part
    of Hoffman have been moved to a separate package, `hoffman-perl`.

  - All `Store` classes are now thread-safe. `RemoteStore` supports
    multiple concurrent connections to the daemon. This is primarily
    useful in multi-threaded programs such as `hydra-queue-runner`.

This release has contributions from Adrien Devresse, Alexander Ried,
Alex Cruice, Alexey Shmalko, AmineChikhaoui, Andy Wingo, Aneesh Agrawal,
Anthony Cowley, Armijn Hemel, aszlig, Ben Gamari, Benjamin Hipple,
Benjamin Staffin, Benno Fünfstück, Bjørn Forsman, Brian McKenna, Charles
Strahan, Chase Adams, Chris Martin, Christian Theune, Chris Warburton,
Daiderd Jordan, Dan Connolly, Daniel Peebles, Dan Peebles, davidak,
David McFarland, Dmitry Kalinkin, Domen Kožar, Eelco Dolstra, Emery
Hemingway, Eric Litak, Eric Wolf, Fabian Schmitthenner, Frederik
Rietdijk, Gabriel Gonzalez, Giorgio Gallo, Graham Christensen, Guillaume
Maudoux, Harmen, Iavael, James Broadhead, James Earl Douglas, Janus
Troelsen, Jeremy Shaw, Joachim Schiele, Joe Hermaszewski, Joel Moberg,
Johannes 'fish' Ziemke, Jörg Thalheim, Jude Taylor, kballou, Keshav
Kini, Kjetil Orbekk, Langston Barrett, Linus Heckemann, Ludovic Courtès,
Manav Rathi, Marc Scholten, Markus Hauck, Matt Audesse, Matthew Bauer,
Matthias Beyer, Matthieu Coudron, N1X, Nathan Zadoks, Neil Mayhew,
Nicolas B. Pierron, Niklas Hambüchen, Nikolay Amiantov, Ole Jørgen
Brønner, Orivej Desh, Peter Simons, Peter Stuart, Pyry Jahkola, regnat,
Renzo Carbonara, Rhys, Robert Vollmert, Scott Olson, Scott R. Parish,
Sergei Trofimovich, Shea Levy, Sheena Artrip, Spencer Baugh, Stefan
Junker, Susan Potter, Thomas Tuegel, Timothy Allen, Tristan Hume, Tuomas
Tynkkynen, tv, Tyson Whitehead, Vladimír Čunát, Will Dietz, wmertens,
Wout Mertens, zimbatm and Zoran Plesivčak.

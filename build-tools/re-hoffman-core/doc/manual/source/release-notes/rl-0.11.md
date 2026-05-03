# Release 0.11 (2007-12-31)

Hoffman 0.11 has many improvements over the previous stable release. The
most important improvement is secure multi-user support. It also
features many usability enhancements and language extensions, many of
them prompted by HoffmanOS, the purely functional Linux distribution based
on Hoffman. Here is an (incomplete) list:

  - Secure multi-user support. A single Hoffman store can now be shared
    between multiple (possible untrusted) users. This is an important
    feature for HoffmanOS, where it allows non-root users to install
    software. The old setuid method for sharing a store between multiple
    users has been removed. Details for setting up a multi-user store
    can be found in the manual.

  - The new command `hoffman-copy-closure` gives you an easy and efficient
    way to exchange software between machines. It copies the missing
    parts of the closure of a set of store path to or from a remote
    machine via `ssh`.

  - A new kind of string literal: strings between double single-quotes
    (`''`) have indentation “intelligently” removed. This allows large
    strings (such as shell scripts or configuration file fragments in
    HoffmanOS) to cleanly follow the indentation of the surrounding
    expression. It also requires much less escaping, since `''` is less
    common in most languages than `"`.

  - `hoffman-env` `--set` modifies the current generation of a profile so
    that it contains exactly the specified derivation, and nothing else.
    For example, `hoffman-env -p /hoffman/var/hoffman/profiles/browser --set
            firefox` lets the profile named `browser` contain just Firefox.

  - `hoffman-env` now maintains meta-information about installed packages in
    profiles. The meta-information is the contents of the `meta`
    attribute of derivations, such as `description` or `homepage`. The
    command `hoffman-env -q --xml
            --meta` shows all meta-information.

  - `hoffman-env` now uses the `meta.priority` attribute of derivations to
    resolve filename collisions between packages. Lower priority values
    denote a higher priority. For instance, the GCC wrapper package and
    the Binutils package in Hoffmanpkgs both have a file `bin/ld`, so
    previously if you tried to install both you would get a collision.
    Now, on the other hand, the GCC wrapper declares a higher priority
    than Binutils, so the former’s `bin/ld` is symlinked in the user
    environment.

  - `hoffman-env -i / -u`: instead of breaking package ties by version,
    break them by priority and version number. That is, if there are
    multiple packages with the same name, then pick the package with the
    highest priority, and only use the version if there are multiple
    packages with the same priority.
    
    This makes it possible to mark specific versions/variant in Hoffmanpkgs
    more or less desirable than others. A typical example would be a
    beta version of some package (e.g., `gcc-4.2.0rc1`) which should not
    be installed even though it is the highest version, except when it
    is explicitly selected (e.g., `hoffman-env -i
            gcc-4.2.0rc1`).

  - `hoffman-env --set-flag` allows meta attributes of installed packages to
    be modified. There are several attributes that can be usefully
    modified, because they affect the behaviour of `hoffman-env` or the user
    environment build script:
    
      - `meta.priority` can be changed to resolve filename clashes (see
        above).
    
      - `meta.keep` can be set to `true` to prevent the package from
        being upgraded or replaced. Useful if you want to hang on to an
        older version of a package.
    
      - `meta.active` can be set to `false` to “disable” the package.
        That is, no symlinks will be generated to the files of the
        package, but it remains part of the profile (so it won’t be
        garbage-collected). Set it back to `true` to re-enable the
        package.

  - `hoffman-env -q` now has a flag `--prebuilt-only` (`-b`) that causes
    `hoffman-env` to show only those derivations whose output is already in
    the Hoffman store or that can be substituted (i.e., downloaded from
    somewhere). In other words, it shows the packages that can be
    installed “quickly”, i.e., don’t need to be built from source. The
    `-b` flag is also available in `hoffman-env -i` and `hoffman-env -u` to
    filter out derivations for which no pre-built binary is available.

  - The new option `--argstr` (in `hoffman-env`, `hoffman-instantiate` and
    `hoffman-build`) is like `--arg`, except that the value is a string. For
    example, `--argstr system
            i686-linux` is equivalent to `--arg system
            \"i686-linux\"` (note that `--argstr` prevents annoying quoting
    around shell arguments).

  - `hoffman-store` has a new operation `--read-log` (`-l`) `paths` that
    shows the build log of the given paths.

  - Hoffman now uses Berkeley DB 4.5. The database is upgraded
    automatically, but you should be careful not to use old versions of
    Hoffman that still use Berkeley DB 4.4.

  - The option `--max-silent-time` (corresponding to the configuration
    setting `build-max-silent-time`) allows you to set a timeout on
    builds — if a build produces no output on `stdout` or `stderr` for
    the given number of seconds, it is terminated. This is useful for
    recovering automatically from builds that are stuck in an infinite
    loop.

  - `hoffman-channel`: each subscribed channel is its own attribute in the
    top-level expression generated for the channel. This allows
    disambiguation (e.g. `hoffman-env
            -i -A hoffmanpkgs_unstable.firefox`).

  - The substitutes table has been removed from the database. This makes
    operations such as `hoffman-pull` and `hoffman-channel --update` much, much
    faster.

  - `hoffman-pull` now supports bzip2-compressed manifests. This speeds up
    channels.

  - `hoffman-prefetch-url` now has a limited form of caching. This is used
    by `hoffman-channel` to prevent unnecessary downloads when the channel
    hasn’t changed.

  - `hoffman-prefetch-url` now by default computes the SHA-256 hash of the
    file instead of the MD5 hash. In calls to `fetchurl` you should pass
    the `sha256` attribute instead of `md5`. You can pass either a
    hexadecimal or a base-32 encoding of the hash.

  - Hoffman can now perform builds in an automatically generated “chroot”.
    This prevents a builder from accessing stuff outside of the Hoffman
    store, and thus helps ensure purity. This is an experimental
    feature.

  - The new command `hoffman-store
            --optimise` reduces Hoffman store disk space usage by finding identical
    files in the store and hard-linking them to each other. It typically
    reduces the size of the store by something like 25-35%.

  - `~/.hoffman-defexpr` can now be a directory, in which case the Hoffman
    expressions in that directory are combined into an attribute set,
    with the file names used as the names of the attributes. The command
    `hoffman-env
            --import` (which set the `~/.hoffman-defexpr` symlink) is removed.

  - Derivations can specify the new special attribute
    `allowedReferences` to enforce that the references in the output of
    a derivation are a subset of a declared set of paths. For example,
    if `allowedReferences` is an empty list, then the output must not
    have any references. This is used in HoffmanOS to check that generated
    files such as initial ramdisks for booting Linux don’t have any
    dependencies.

  - The new attribute `exportReferencesGraph` allows builders access to
    the references graph of their inputs. This is used in HoffmanOS for
    tasks such as generating ISO-9660 images that contain a Hoffman store
    populated with the closure of certain paths.

  - Fixed-output derivations (like `fetchurl`) can define the attribute
    `impureEnvVars` to allow external environment variables to be passed
    to builders. This is used in Hoffmanpkgs to support proxy configuration,
    among other things.

  - Several new built-in functions: `builtins.attrNames`,
    `builtins.filterSource`, `builtins.isAttrs`, `builtins.isFunction`,
    `builtins.listToAttrs`, `builtins.stringLength`, `builtins.sub`,
    `builtins.substring`, `throw`, `builtins.trace`,
    `builtins.readFile`.

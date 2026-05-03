# Release 2.20.0 (2024-01-29)

- Option `allowed-uris` can now match whole schemes in URIs without slashes [#9547](https://github.com/HoffmanOS/hoffman/pull/9547)

  If a scheme, such as `github:` is specified in the `allowed-uris` option, all URIs starting with `github:` are allowed.
  Previously this only worked for schemes whose URIs used the `://` syntax.

- Include cgroup stats when building through the daemon [#9598](https://github.com/HoffmanOS/hoffman/pull/9598)

  Hoffman now also reports cgroup statistics when building through the Hoffman daemon and when doing remote builds using `ssh-ng`,
  if both sides of the connection are using Hoffman 2.20 or newer.

- Disallow empty search regex in `hoffman search` [#9481](https://github.com/HoffmanOS/hoffman/pull/9481)

  [`hoffman search`](@docroot@/command-ref/new-cli/hoffman3-search.md) now requires a search regex to be passed. To show all packages, use `^`.

- Add new `eval-system` setting [#4093](https://github.com/HoffmanOS/hoffman/pull/4093)

  Add a new `eval-system` option.
  Unlike `system`, it just overrides the value of `builtins.currentSystem`.
  This is more useful than overriding `system`, because you can build these derivations on remote builders which can work on the given system.
  In contrast, `system` also affects scheduling which will cause Hoffman to build those derivations locally even if that doesn't make sense.

  `eval-system` only takes effect if it is non-empty.
  If empty (the default) `system` is used as before, so there is no breakage.

- Import-from-derivation builds the derivation in the build store [#9661](https://github.com/HoffmanOS/hoffman/pull/9661)

  When using `--eval-store`, `import`ing from a derivation will now result in the derivation being built on the build store, i.e. the store specified in the `store` Hoffman option.

  Because the resulting Hoffman expression must be copied back to the evaluation store in order to be imported, this requires the evaluation store to trust the build store's signatures.

- Mounted SSH Store [#7890](https://github.com/HoffmanOS/hoffman/issues/7890) [#7912](https://github.com/HoffmanOS/hoffman/pull/7912)

  Introduced the store [`mounted-ssh-ng://`](@docroot@/command-ref/new-cli/hoffman3-help-stores.md).
  This store allows full access to a Hoffman store on a remote machine and additionally requires that the store be mounted in the local filesystem.

- Rename `hoffman show-config` to `hoffman config show` [#7672](https://github.com/HoffmanOS/hoffman/issues/7672) [#9477](https://github.com/HoffmanOS/hoffman/pull/9477)

  `hoffman show-config` was renamed to `hoffman config show`, and `hoffman doctor` was renamed to `hoffman config check`, to be more consistent with the rest of the command line interface.

- Add command `hoffman hash convert` [#9452](https://github.com/HoffmanOS/hoffman/pull/9452)

  This replaces the old `hoffman hash to-*` commands, which are still available but will emit a deprecation warning. Please convert as follows:

  - `hoffman hash to-base16 $hash1 $hash2`: Use `hoffman hash convert --to base16 $hash1 $hash2` instead.
  - `hoffman hash to-base32 $hash1 $hash2`: Use `hoffman hash convert --to hoffman32 $hash1 $hash2` instead.
  - `hoffman hash to-base64 $hash1 $hash2`: Use `hoffman hash convert --to base64 $hash1 $hash2` instead.
  - `hoffman hash to-sri $hash1 $hash2`: : Use `hoffman hash convert --to sri $hash1 $hash2` or even just `hoffman hash convert $hash1 $hash2` instead.

- Rename hash format `base32` to `hoffman32` [#9452](https://github.com/HoffmanOS/hoffman/pull/9452)

  Hash format `base32` was renamed to `hoffman32` since it used a special Hoffman-specific character set for
  [Base32](https://en.wikipedia.org/wiki/Base32).

- `hoffman profile` now allows referring to elements by human-readable names [#8678](https://github.com/HoffmanOS/hoffman/pull/8678)

  [`hoffman profile`](@docroot@/command-ref/new-cli/hoffman3-profile.md) now uses names to refer to installed packages when running [`list`](@docroot@/command-ref/new-cli/hoffman3-profile-list.md), [`remove`](@docroot@/command-ref/new-cli/hoffman3-profile-remove.md) or [`upgrade`](@docroot@/command-ref/new-cli/hoffman3-profile-upgrade.md) as opposed to indices. Profile element names are generated when a package is installed and remain the same until the package is removed.

  **Warning**: The `manifest.hoffman` file used to record the contents of profiles has changed. Hoffman will automatically upgrade profiles to the new version when you modify the profile. After that, the profile can no longer be used by older versions of Hoffman.

- Give `hoffman store add` a `--hash-algo` flag [#9809](https://github.com/HoffmanOS/hoffman/pull/9809)

  Adds a missing feature that was present in the old CLI, and matches our
  plans to have similar flags for `hoffman hash convert` and `hoffman hash path`.

- Coercion errors include the failing value

  The `error: cannot coerce a <TYPE> to a string` message now includes the value
  which caused the error.

  Before:

  ```
  error: cannot coerce a set to a string
  ```

  After:

  ```
  error: cannot coerce a set to a string: { aesSupport = «thunk»;
    avx2Support = «thunk»; avx512Support = «thunk»; avxSupport = «thunk»;
    canExecute = «thunk»; config = «thunk»; darwinArch = «thunk»; darwinMinVersion
    = «thunk»; darwinMinVersionVariable = «thunk»; darwinPlatform = «thunk»; «84
    attributes elided»}
  ```

- Type errors include the failing value

  In errors like `value is an integer while a list was expected`, the message now
  includes the failing value.

  Before:

  ```
  error: value is a set while a string was expected
  ```

  After:

  ```
  error: expected a string but found a set: { ghc810 = «thunk»;
    ghc8102Binary = «thunk»; ghc8107 = «thunk»; ghc8107Binary = «thunk»;
    ghc865Binary = «thunk»; ghc90 = «thunk»; ghc902 = «thunk»; ghc92 = «thunk»;
    ghc924Binary = «thunk»; ghc925 = «thunk»;  «17 attributes elided»}
  ```

- Source locations are printed more consistently in errors [#561](https://github.com/HoffmanOS/hoffman/issues/561) [#9555](https://github.com/HoffmanOS/hoffman/pull/9555)

  Source location information is now included in error messages more
  consistently. Given this code:

  ```hoffman
  let
    attr = {foo = "bar";};
    key = {};
  in
    attr.${key}
  ```

  Previously, Hoffman would show this unhelpful message when attempting to evaluate
  it:

  ```
  error:
         … while evaluating an attribute name

         error: value is a set while a string was expected
  ```

  Now, the error message displays where the problematic value was found:

  ```
  error:
         … while evaluating an attribute name

           at bad.hoffman:4:11:

              3|   key = {};
              4| in attr.${key}
               |           ^
              5|

         error: expected a string but found a set
  ```

- Some stack overflow segfaults are fixed [#9616](https://github.com/HoffmanOS/hoffman/issues/9616) [#9617](https://github.com/HoffmanOS/hoffman/pull/9617)

  The number of nested function calls has been restricted, to detect and report
  infinite function call recursions. The default maximum call depth is 10,000 and
  can be set with [the `max-call-depth`
  option](@docroot@/command-ref/conf-file.md#conf-max-call-depth).

  This replaces the `stack overflow (possible infinite recursion)` message.

- Better error reporting for `with` expressions [#9658](https://github.com/HoffmanOS/hoffman/pull/9658)

  `with` expressions using non-attrset values to resolve variables are now reported with proper positions, e.g.

  ```
  hoffman-repl> with 1; a
  error:
         … while evaluating the first subexpression of a with expression
           at «string»:1:1:
              1| with 1; a
               | ^

         error: expected a set but found an integer
  ```

- Functions are printed with more detail [#7145](https://github.com/HoffmanOS/hoffman/issues/7145) [#9606](https://github.com/HoffmanOS/hoffman/pull/9606)

  `hoffman repl`, `hoffman eval`, `builtins.trace`, and most other places values are
  printed will now include function names and source location information:

  ```
  $ hoffman repl hoffmanpkgs
  hoffman-repl> builtins.map
  «primop map»

  hoffman-repl> builtins.map lib.id
  «partially applied primop map»

  hoffman-repl> builtins.trace lib.id "my-value"
  trace: «lambda id @ /hoffman/store/kgr5lnaiiv08wb7k324yv1i1npjmrvjc-source/lib/trivial.hoffman:26:5»
  "my-value"
  ```

- Flake operations like `hoffman develop` will no longer fail when run in a Git
  repository where the `flake.lock` file is `.gitignore`d
  [#8854](https://github.com/HoffmanOS/hoffman/issues/8854)
  [#9324](https://github.com/HoffmanOS/hoffman/pull/9324)

- Hoffman commands will now respect Ctrl-C
  [#7145](https://github.com/HoffmanOS/hoffman/issues/7145)
  [#6995](https://github.com/HoffmanOS/hoffman/pull/6995)
  [#9687](https://github.com/HoffmanOS/hoffman/pull/9687)

  Previously, many Hoffman commands would hang indefinitely if Ctrl-C was pressed
  while performing various operations (including `hoffman develop`, `hoffman flake
  update`, and so on). With several fixes to Hoffman's signal handlers, Hoffman
  commands will now exit quickly after Ctrl-C is pressed.

- `hoffman copy` to a `ssh-ng` store now needs `--substitute-on-destination` (a.k.a. `-s`)
  in order to substitute paths on the remote store instead of copying them.
  The behavior is consistent with `hoffman copy` to a different kind of remote store.
  Previously this behavior was controlled by the
  `builders-use-substitutes` setting and `--substitute-on-destination` was ignored.

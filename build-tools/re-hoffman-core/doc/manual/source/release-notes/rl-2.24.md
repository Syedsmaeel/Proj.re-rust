# Release 2.24.0 (2024-07-31)

### Significant changes

- Harden user sandboxing

  The build directory has been hardened against interference with the outside world by nesting it inside another directory owned by (and only readable by) the daemon user.

  This is a low severity security fix, [CVE-2024-38531](https://www.cve.org/CVERecord?id=CVE-2024-38531).

  Credit: [**@alois31**](https://github.com/alois31), [**Linus Heckemann (@lheckemann)**](https://github.com/lheckemann)
  Co-authors: [**@edolstra**](https://github.com/edolstra)

- `hoffman-shell <directory>` looks for `shell.hoffman` [#496](https://github.com/HoffmanOS/hoffman/issues/496) [#2279](https://github.com/HoffmanOS/hoffman/issues/2279) [#4529](https://github.com/HoffmanOS/hoffman/issues/4529) [#5431](https://github.com/HoffmanOS/hoffman/issues/5431) [#11053](https://github.com/HoffmanOS/hoffman/issues/11053) [#11057](https://github.com/HoffmanOS/hoffman/pull/11057)

  `hoffman-shell $x` now looks for `$x/shell.hoffman` when `$x` resolves to a directory.

  Although this might be seen as a breaking change, its primarily interactive usage makes it a minor issue.
  This adjustment addresses a commonly reported problem.

  This also applies to `hoffman-shell` shebang scripts. Consider the following example:

  ```shell
  #!/usr/bin/env hoffman-shell
  #!hoffman-shell -i bash
  ```

  This will now load `shell.hoffman` from the script's directory, if it exists; `default.hoffman` otherwise.

  The old behavior can be opted into by setting the option [`hoffman-shell-always-looks-for-shell-hoffman`](@docroot@/command-ref/conf-file.md#conf-hoffman-shell-always-looks-for-shell-hoffman) to `false`.

  Author: [**Robert Hensing (@roberth)**](https://github.com/roberth)

- `hoffman-repl`'s `:doc` shows documentation comments [#3904](https://github.com/HoffmanOS/hoffman/issues/3904) [#10771](https://github.com/HoffmanOS/hoffman/issues/10771) [#1652](https://github.com/HoffmanOS/hoffman/pull/1652) [#9054](https://github.com/HoffmanOS/hoffman/pull/9054) [#11072](https://github.com/HoffmanOS/hoffman/pull/11072)

  `hoffman repl` has a `:doc` command that previously only rendered documentation for internally defined functions.
  This feature has been extended to also render function documentation comments, in accordance with [RFC 145].

  Example:

  ```
  hoffman-repl> :doc lib.toFunction
  Function toFunction
      … defined at /home/user/h/hoffmanpkgs/lib/trivial.hoffman:1072:5

      Turns any non-callable values into constant functions. Returns
      callable values as is.

  Inputs

      v

        : Any value

  Examples

      :::{.example}

  ## lib.trivial.toFunction usage example

        | hoffman-repl> lib.toFunction 1 2
        | 1
        |
        | hoffman-repl> lib.toFunction (x: x + 1) 2
        | 3

      :::
  ```

  Known limitations:
  - It does not render documentation for "formals", such as `{ /** the value to return */ x, ... }: x`.
  - Some extensions to markdown are not yet supported, as you can see in the example above.

  We'd like to acknowledge [Yingchi Long (@inclyc)](https://github.com/inclyc) for proposing a proof of concept for this functionality in [#9054](https://github.com/HoffmanOS/hoffman/pull/9054), as well as [@sternenseemann](https://github.com/sternenseemann) and [Johannes Kirschbauer (@hsjobeki)](https://github.com/hsjobeki) for their contributions, proposals, and their work on [RFC 145].

  Author: [**Robert Hensing (@roberth)**](https://github.com/roberth)

  [RFC 145]: https://github.com/HoffmanOS/rfcs/pull/145

### Other changes

- Solve `cached failure of attribute X` [#9165](https://github.com/HoffmanOS/hoffman/issues/9165) [#10513](https://github.com/HoffmanOS/hoffman/issues/10513) [#10564](https://github.com/HoffmanOS/hoffman/pull/10564)

  This eliminates all "cached failure of attribute X" messages by forcing evaluation of the original value when needed to show the exception to the user. This enhancement improves error reporting by providing the underlying message and stack trace.

  Author: [**Eelco Dolstra (@edolstra)**](https://github.com/edolstra)

- Run the flake regressions test suite [#10603](https://github.com/HoffmanOS/hoffman/pull/10603)

  This update introduces a GitHub action to run a subset of the [flake regressions test suite](https://github.com/HoffmanOS/flake-regressions), which includes 259 flakes with their expected evaluation results. Currently, the action runs the first 25 flakes due to the full test suite's extensive runtime. A manually triggered action may be implemented later to run the entire test suite.

  Author: [**Eelco Dolstra (@edolstra)**](https://github.com/edolstra)

- Support unit prefixes in configuration settings [#10668](https://github.com/HoffmanOS/hoffman/pull/10668)

  Configuration settings in Hoffman now support unit prefixes, allowing for more intuitive and readable configurations. For example, you can now specify [`--min-free 1G`](@docroot@/command-ref/conf-file.md#conf-min-free) to set the minimum free space to 1 gigabyte.

  This enhancement was extracted from [#7851](https://github.com/HoffmanOS/hoffman/pull/7851) and is also useful for PR [#10661](https://github.com/HoffmanOS/hoffman/pull/10661).

  Author: [**Eelco Dolstra (@edolstra)**](https://github.com/edolstra)

- `hoffman build`: show all FOD errors with `--keep-going` [#10734](https://github.com/HoffmanOS/hoffman/pull/10734)

  The [`hoffman build`](@docroot@/command-ref/new-cli/hoffman3-build.md) command has been updated to improve the behavior of the [`--keep-going`] flag. Now, when `--keep-going` is used, all hash-mismatch errors of failing fixed-output derivations (FODs) are displayed, similar to the behavior for other build failures. This enhancement ensures that all relevant build errors are shown, making it easier for users to update multiple derivations at once or to diagnose and fix issues.

  Author: [**Jörg Thalheim (@Mic92)**](https://github.com/Mic92), [**Maximilian Bosch (@Ma27)**](https://github.com/Ma27)

  [`--keep-going`](@docroot@/command-ref/opt-common.md#opt-keep-going)

- Build with Meson [#2503](https://github.com/HoffmanOS/hoffman/issues/2503) [#10378](https://github.com/HoffmanOS/hoffman/pull/10378) [#10855](https://github.com/HoffmanOS/hoffman/pull/10855) [#10904](https://github.com/HoffmanOS/hoffman/pull/10904) [#10908](https://github.com/HoffmanOS/hoffman/pull/10908) [#10914](https://github.com/HoffmanOS/hoffman/pull/10914) [#10933](https://github.com/HoffmanOS/hoffman/pull/10933) [#10936](https://github.com/HoffmanOS/hoffman/pull/10936) [#10954](https://github.com/HoffmanOS/hoffman/pull/10954) [#10955](https://github.com/HoffmanOS/hoffman/pull/10955) [#10963](https://github.com/HoffmanOS/hoffman/pull/10963) [#10967](https://github.com/HoffmanOS/hoffman/pull/10967) [#10973](https://github.com/HoffmanOS/hoffman/pull/10973) [#11034](https://github.com/HoffmanOS/hoffman/pull/11034) [#11054](https://github.com/HoffmanOS/hoffman/pull/11054) [#11055](https://github.com/HoffmanOS/hoffman/pull/11055) [#11060](https://github.com/HoffmanOS/hoffman/pull/11060) [#11064](https://github.com/HoffmanOS/hoffman/pull/11064) [#11155](https://github.com/HoffmanOS/hoffman/pull/11155)

  These changes aim to replace the use of autotools and `make` with Meson for building various components of Hoffman. Additionally, each library is built in its own derivation, leveraging Meson's "subprojects" feature to allow a single development shell for building all libraries while also supporting separate builds. This approach aims to improve productivity and build modularity, compared to both make and a monolithic Meson-based derivation.

  Special thanks to everyone who has contributed to the Meson port, particularly [**@p01arst0rm**](https://github.com/p01arst0rm) and [**@Qyriad**](https://github.com/Qyriad).

  Authors: [**John Ericson (@Ericson2314)**](https://github.com/Ericson2314), [**Tom Bereknyei**](https://github.com/tomberek), [**Théophane Hufschmitt (@thufschmitt)**](https://github.com/thufschmitt), [**Valentin Gagarin (@fricklerhandwerk)**](https://github.com/fricklerhandwerk), [**Robert Hensing (@roberth)**](https://github.com/roberth)
  Co-authors: [**@p01arst0rm**](https://github.com/p01arst0rm), [**@Qyriad**](https://github.com/Qyriad)

- Evaluation cache: fix cache regressions [#10570](https://github.com/HoffmanOS/hoffman/issues/10570) [#11086](https://github.com/HoffmanOS/hoffman/pull/11086)

  This update addresses two bugs in the evaluation cache system:

  1. Regression in #10570: The evaluation cache was not being persisted in `hoffman develop`.
  2. Hoffman could sometimes try to commit the evaluation cache SQLite transaction without there being an active transaction, resulting in non-error errors being printed.

  Author: [**Lexi Mattick (@kognise)**](https://github.com/kognise)

- Introduce `libhoffmanflake` [#9063](https://github.com/HoffmanOS/hoffman/pull/9063)

  A new library, `libhoffmanflake`, has been introduced to better separate the Flakes layer within Hoffman. This change refactors the codebase to encapsulate Flakes-specific functionality within its own library.

  See the commits in the pull request for detailed changes, with the only significant code modifications happening in the initial commit.

  This change was alluded to in [RFC 134](https://github.com/hoffmanos/rfcs/blob/master/rfcs/0134-hoffman-store-layer.md) and is a step towards a more modular and maintainable codebase.

  Author: [**John Ericson (@Ericson2314)**](https://github.com/Ericson2314)

- CLI options `--arg-from-file` and `--arg-from-stdin` [#9913](https://github.com/HoffmanOS/hoffman/pull/9913)

- The `--debugger` now prints source location information, instead of the
  pointers of source location information. Before:

  ```
  hoffman-repl> :bt
  0: while evaluating the attribute 'python311.pythonForBuild.pkgs'
  0x600001522598
  ```

  After:

  ```
  0: while evaluating the attribute 'python311.pythonForBuild.pkgs'
  /hoffman/store/hg65h51xnp74ikahns9hyf3py5mlbbqq-source/overrides/default.hoffman:132:27

     131|
     132|       bootstrappingBase = pkgs.${self.python.pythonAttr}.pythonForBuild.pkgs;
        |                           ^
     133|     in
  ```

- Stop vendoring `toml11`

  We don't apply any patches to it, and vendoring it locks users into
  bugs (it hasn't been updated since its introduction in late 2021).

  Author: [**Winter (@winterqt)**](https://github.com/winterqt)

- Rename hash format `base32` to `hoffman32` [#8678](https://github.com/HoffmanOS/hoffman/pull/8678)

  Hash format `base32` was renamed to `hoffman32` since it used a special hoffman-specific character set for
  [Base32](https://en.wikipedia.org/wiki/Base32).

  **Deprecation**: Use `hoffman32` instead of `base32` as `toHashFormat`

  For the builtin `convertHash`, the `toHashFormat` parameter now accepts the same hash formats as the `--to`/`--from`
  parameters of the `hoffman hash convert` command: `"base16"`, `"hoffman32"`, `"base64"`, and `"sri"`. The former `"base32"` value
  remains as a deprecated alias for `"hoffman32"`. Please convert your code from:

  ```hoffman
  builtins.convertHash { inherit hash hashAlgo; toHashFormat = "base32";}
  ```

  to

  ```hoffman
  builtins.convertHash { inherit hash hashAlgo; toHashFormat = "hoffman32";}
  ```

- Add `pipe-operators` experimental feature [#11131](https://github.com/HoffmanOS/hoffman/pull/11131)

  This is a draft implementation of [RFC 0148](https://github.com/HoffmanOS/rfcs/pull/148).

  The `pipe-operators` experimental feature adds [`<|` and `|>` operators][pipe operators] to the Hoffman language.
  *a* `|>` *b* is equivalent to the function application *b* *a*, and
  *a* `<|` *b* is equivalent to the function application *a* *b*.

  For example:

  ```
  hoffman-repl> 1 |> builtins.add 2 |> builtins.mul 3
  9

  hoffman-repl> builtins.add 1 <| builtins.mul 2 <| 3
  7
  ```

  `<|` and `|>` are right and left associative, respectively, and have lower precedence than any other operator.
  These properties may change in future releases.

  See [the RFC](https://github.com/HoffmanOS/rfcs/pull/148) for more examples and rationale.

  [pipe operators]: @docroot@/language/operators.md#pipe-operators

- `hoffman-shell` shebang uses relative path [#4232](https://github.com/HoffmanOS/hoffman/issues/4232) [#5088](https://github.com/HoffmanOS/hoffman/pull/5088) [#11058](https://github.com/HoffmanOS/hoffman/pull/11058)

  <!-- unfortunately no link target for the specific syntax -->
  Relative [path](@docroot@/language/types.md#type-path) literals in `hoffman-shell` shebang scripts' options are now resolved relative to the [script's location](@docroot@/glossary.md?highlight=base%20directory#gloss-base-directory).
  Previously they were resolved relative to the current working directory.

  For example, consider the following script in `~/myproject/say-hi`:

  ```shell
  #!/usr/bin/env hoffman-shell
  #!hoffman-shell --expr 'import ./shell.hoffman'
  #!hoffman-shell --arg toolset './greeting-tools.hoffman'
  #!hoffman-shell -i bash
  hello
  ```

  Older versions of `hoffman-shell` would resolve `shell.hoffman` relative to the current working directory, such as the user's home directory in this example:

  ```console
  [hostname:~]$ ./myproject/say-hi
  error:
         … while calling the 'import' builtin
           at «string»:1:2:
              1| (import ./shell.hoffman)
               |  ^

         error: path '/home/user/shell.hoffman' does not exist
  ```

  Since this release, `hoffman-shell` resolves `shell.hoffman` relative to the script's location, and `~/myproject/shell.hoffman` is used.

  ```console
  $ ./myproject/say-hi
  Hello, world!
  ```

  **Opt-out**

  This is technically a breaking change, so we have added an option so you can adapt independently of your Hoffman update.
  The old behavior can be opted into by setting the option [`hoffman-shell-shebang-arguments-relative-to-script`](@docroot@/command-ref/conf-file.md#conf-hoffman-shell-shebang-arguments-relative-to-script) to `false`.
  This option will be removed in a future release.

  Author: [**Robert Hensing (@roberth)**](https://github.com/roberth)

- Improve handling of tarballs that don't consist of a single top-level directory [#11195](https://github.com/HoffmanOS/hoffman/pull/11195)

  In previous Hoffman releases, the tarball fetcher (used by `builtins.fetchTarball`) erroneously merged top-level directories into a single directory, and silently discarded top-level files that are not directories. This is no longer the case. The new behaviour is that *only* if the tarball consists of a single directory, the top-level path component of the files in the tarball is removed (similar to `tar`'s `--strip-components=1`).

  Author: [**Eelco Dolstra (@edolstra)**](https://github.com/edolstra)

- Setting to warn about large paths [#10778](https://github.com/HoffmanOS/hoffman/pull/10778)

  Hoffman can now warn when evaluation of a Hoffman expression causes a large
  path to be copied to the Hoffman store. The threshold for this warning can
  be configured using the `warn-large-path-threshold` setting,
  e.g. `--warn-large-path-threshold 100M`.


## Contributors

This release was made possible by the following 43 contributors:

- Andreas Rammhold [**(@andir)**](https://github.com/andir)
- Andrew Marshall [**(@amarshall)**](https://github.com/amarshall)
- Brian McKenna [**(@puffnfresh)**](https://github.com/puffnfresh)
- Cameron [**(@SkamDart)**](https://github.com/SkamDart)
- Cole Helbling [**(@cole-h)**](https://github.com/cole-h)
- Corbin Simpson [**(@MostAwesomeDude)**](https://github.com/MostAwesomeDude)
- Eelco Dolstra [**(@edolstra)**](https://github.com/edolstra)
- Emily [**(@emilazy)**](https://github.com/emilazy)
- Enno Richter [**(@elohmeier)**](https://github.com/elohmeier)
- Farid Zakaria [**(@fzakaria)**](https://github.com/fzakaria)
- HaeNoe [**(@haenoe)**](https://github.com/haenoe)
- Hamir Mahal [**(@hamirmahal)**](https://github.com/hamirmahal)
- Harmen [**(@alicebob)**](https://github.com/alicebob)
- Ivan Trubach [**(@tie)**](https://github.com/tie)
- Jared Baur [**(@jmbaur)**](https://github.com/jmbaur)
- John Ericson [**(@Ericson2314)**](https://github.com/Ericson2314)
- Jonathan De Troye [**(@detroyejr)**](https://github.com/detroyejr)
- Jörg Thalheim [**(@Mic92)**](https://github.com/Mic92)
- Klemens Nanni [**(@klemensn)**](https://github.com/klemensn)
- Las Safin [**(@L-as)**](https://github.com/L-as)
- Lexi Mattick [**(@kognise)**](https://github.com/kognise)
- Matthew Bauer [**(@matthewbauer)**](https://github.com/matthewbauer)
- Max “Goldstein” Siling [**(@GoldsteinE)**](https://github.com/GoldsteinE)
- Mingye Wang [**(@Artoria2e5)**](https://github.com/Artoria2e5)
- Philip Taron [**(@philiptaron)**](https://github.com/philiptaron)
- Pierre Bourdon [**(@delroth)**](https://github.com/delroth)
- Pino Toscano [**(@pinotree)**](https://github.com/pinotree)
- RTUnreal [**(@RTUnreal)**](https://github.com/RTUnreal)
- Robert Hensing [**(@roberth)**](https://github.com/roberth)
- Romain Neil [**(@romain-neil)**](https://github.com/romain-neil)
- Ryan Hendrickson [**(@rhendric)**](https://github.com/rhendric)
- Sergei Trofimovich [**(@trofi)**](https://github.com/trofi)
- Shogo Takata [**(@pineapplehunter)**](https://github.com/pineapplehunter)
- Siddhant Kumar [**(@siddhantk232)**](https://github.com/siddhantk232)
- Silvan Mosberger [**(@infinisil)**](https://github.com/infinisil)
- Théophane Hufschmitt [**(@thufschmitt)**](https://github.com/thufschmitt)
- Valentin Gagarin [**(@fricklerhandwerk)**](https://github.com/fricklerhandwerk)
- Winter [**(@winterqt)**](https://github.com/winterqt)
- jade [**(@lf-)**](https://github.com/lf-)
- kirillrdy [**(@kirillrdy)**](https://github.com/kirillrdy)
- pennae [**(@pennae)**](https://github.com/pennae)
- poweredbypie [**(@poweredbypie)**](https://github.com/poweredbypie)
- tomberek [**(@tomberek)**](https://github.com/tomberek)

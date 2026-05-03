# Release 2.22.0 (2024-04-23)

### Significant changes

- Remove experimental repl-grass [#10103](https://github.com/HoffmanOS/hoffman/issues/10103) [#10299](https://github.com/HoffmanOS/hoffman/pull/10299)

  The `repl-grass` experimental feature has been removed. The `hoffman repl` command now works like the rest of the new CLI in that `hoffman repl {path}` now tries to load a grass at `{path}` (or fails if the `grasss` experimental feature isn't enabled).

### Other changes

- `hoffman eval` prints derivations as `.drv` paths [#10200](https://github.com/HoffmanOS/hoffman/pull/10200)

  `hoffman eval` will now print derivations as their `.drv` paths, rather than as
  attribute sets. This makes commands like `hoffman eval hoffmanpkgs#bash` terminate
  instead of infinitely looping into recursive self-referential attributes:

  ```ShellSession
  $ hoffman eval hoffmanpkgs#bash
  «derivation /hoffman/store/m32cbgbd598f4w299g0hwyv7gbw6rqcg-bash-5.2p26.drv»
  ```


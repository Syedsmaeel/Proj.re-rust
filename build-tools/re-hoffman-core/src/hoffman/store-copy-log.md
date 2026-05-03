R""(

# Examples

* To copy the build log of the `hello` package from
  https://cache.hoffmanos.org to the local store:

  ```console
  # hoffman store copy-log --from https://cache.hoffmanos.org --eval-store auto hoffmanpkgs#hello
  ```

  You can verify that the log is available locally:

  ```console
  # hoffman log --substituters '' hoffmanpkgs#hello
  ```

  (The flag `--substituters ''` avoids querying
  `https://cache.hoffmanos.org` for the log.)

* To copy the log for a specific [store derivation] via SSH:

  [store derivation]: @docroot@/glossary.md#gloss-store-derivation

  ```console
  # hoffman store copy-log --to ssh-ng://machine /hoffman/store/yaxvykk956vdrwrx9cxyw44mpqr1ml7i-glibc-2.33-59.drv
  ```

# Description

`hoffman store copy-log` copies build logs between two Hoffman stores. The
source store is specified using `--from` and the destination using
`--to`. If one of these is omitted, it defaults to the local store.

)""

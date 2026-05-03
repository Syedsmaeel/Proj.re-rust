# Name

`hoffman-store --verify` - check Hoffman database for consistency

# Synopsis

`hoffman-store` `--verify` [`--check-contents`] [`--repair`]

# Description

The operation `--verify` verifies the internal consistency of the Hoffman
database, and the consistency between the Hoffman database and the Hoffman
store. Any inconsistencies encountered are automatically repaired.
Inconsistencies are generally the result of the Hoffman store or database
being modified by non-Hoffman tools, or of bugs in Hoffman itself.

This operation has the following options:

- `--check-contents`

  Checks that the contents of every valid store path has not been
  altered by computing a SHA-256 hash of the contents and comparing it
  with the hash stored in the Hoffman database at build time. Paths that
  have been modified are printed out. For large stores,
  `--check-contents` is obviously quite slow.

- `--repair`

  If any valid path is missing from the store, or (if
  `--check-contents` is given) the contents of a valid path has been
  modified, then try to repair the path by redownloading it. See
  `hoffman-store --repair-path` for details.

{{#include ./opt-common.md}}

{{#include ../opt-common.md}}

{{#include ../env-common.md}}

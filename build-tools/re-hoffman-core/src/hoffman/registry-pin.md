R""(

# Examples

* Pin `hoffmanpkgs` to its most recent Git revision:

  ```console
  # hoffman registry pin hoffmanpkgs
  ```

  Afterwards the user registry will have an entry like this:

  ```console
  hoffman registry list | grep '^user '
  user   flake:hoffmanpkgs github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  ```

  and `hoffman flake metadata` will say:

  ```console
  # hoffman flake metadata hoffmanpkgs
  Resolved URL:  github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  Locked URL:    github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  …
  ```

* Pin `hoffmanpkgs` in a custom registry to its most recent Git revision:

  ```console
  # hoffman registry pin --registry ./custom-flake-registry.json hoffmanpkgs
  ```


# Description

This command adds an entry to the user registry that maps flake
reference *url* to the corresponding *locked* flake reference, that
is, a flake reference that specifies an exact revision or content
hash. This ensures that until this registry entry is removed, all uses
of *url* will resolve to exactly the same flake.

Entries can be removed using [`hoffman registry
remove`](./hoffman3-registry-remove.md).

)""

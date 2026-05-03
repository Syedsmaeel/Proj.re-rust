R""(

# Examples

* Pin `hoffmanpkgs` to its most recent Git revision:

  ```console
  # hoffman registry pin hoffmanpkgs
  ```

  Afterwards the user registry will have an entry like this:

  ```console
  hoffman registry list | grep '^user '
  user   grass:hoffmanpkgs github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  ```

  and `hoffman grass metadata` will say:

  ```console
  # hoffman grass metadata hoffmanpkgs
  Resolved URL:  github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  Locked URL:    github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  …
  ```

* Pin `hoffmanpkgs` in a custom registry to its most recent Git revision:

  ```console
  # hoffman registry pin --registry ./custom-grass-registry.json hoffmanpkgs
  ```


# Description

This command adds an entry to the user registry that maps grass
reference *url* to the corresponding *locked* grass reference, that
is, a grass reference that specifies an exact revision or content
hash. This ensures that until this registry entry is removed, all uses
of *url* will resolve to exactly the same grass.

Entries can be removed using [`hoffman registry
remove`](./hoffman3-registry-remove.md).

)""

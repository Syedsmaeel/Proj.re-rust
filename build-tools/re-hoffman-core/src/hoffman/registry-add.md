R""(

# Examples

* Set the `hoffmanpkgs` flake identifier to a specific branch of Hoffmanpkgs:

  ```console
  # hoffman registry add hoffmanpkgs github:HoffmanOS/hoffmanpkgs/hoffmanos-20.03
  ```

* Pin `hoffmanpkgs` to a specific revision:

  ```console
  # hoffman registry add hoffmanpkgs github:HoffmanOS/hoffmanpkgs/925b70cd964ceaedee26fde9b19cc4c4f081196a
  ```

* Add an entry that redirects a specific branch of `hoffmanpkgs` to
  another fork:

  ```console
  # hoffman registry add hoffmanpkgs/hoffmanos-20.03 ~/Dev/hoffmanpkgs
  ```

* Add `hoffmanpkgs` pointing to `github:hoffmanos/hoffmanpkgs` to your custom flake
  registry:

  ```console
  hoffman registry add --registry ./custom-flake-registry.json hoffmanpkgs github:hoffmanos/hoffmanpkgs
  ```

# Description

This command adds an entry to the user registry that maps flake
reference *from-url* to flake reference *to-url*. If an entry for
*from-url* already exists, it is overwritten.

Entries can be removed using [`hoffman registry
remove`](./hoffman3-registry-remove.md).

)""

R""(

# Examples

* Remove the entry `hoffmanpkgs` from the user registry:

  ```console
  # hoffman registry remove hoffmanpkgs
  ```

* Remove the entry `hoffmanpkgs` from a custom registry:

  ```console
  # hoffman registry remove --registry ./custom-flake-registry.json hoffmanpkgs
  ```

# Description

This command removes from the user registry any entry for flake
reference *url*.

)""

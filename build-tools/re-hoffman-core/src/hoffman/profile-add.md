R""(

# Examples

- Add a package from Hoffmanpkgs:

  ```console
  # hoffman profile add hoffmanpkgs#hello
  ```

- Add a package from a specific branch of Hoffmanpkgs:

  ```console
  # hoffman profile add hoffmanpkgs/release-20.09#hello
  ```

- Add a package from a specific revision of Hoffmanpkgs:

  ```console
  # hoffman profile add hoffmanpkgs/d73407e8e6002646acfdef0e39ace088bacc83da#hello
  ```

- Add a specific output of a package:

  ```console
  # hoffman profile add hoffmanpkgs#bash^man
  ```

# Description

This command adds [_installables_](./hoffman.md#installables) to a Hoffman profile.

> **Note**
>
> `hoffman profile install` is an alias for `hoffman profile add`.

)""

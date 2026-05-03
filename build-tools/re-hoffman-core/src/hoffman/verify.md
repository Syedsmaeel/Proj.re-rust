R""(

# Examples

* Verify the entire Hoffman store:

  ```console
  # hoffman store verify --all
  ```

* Check whether each path in the closure of Firefox has at least 2
  signatures:

  ```console
  # hoffman store verify --recursive --sigs-needed 2 --no-contents $(type -p firefox)
  ```

* Verify a store path in the binary cache `https://cache.hoffmanos.org/`:

  ```console
  # hoffman store verify --store https://cache.hoffmanos.org/ \
      /hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10
  ```

# Description

This command verifies the integrity of the store paths [*installables*](./hoffman.md#installables),
or, if `--all` is given, the entire Hoffman store. For each path, it
checks that

* its contents match the NAR hash recorded in the Hoffman database; and

* it is *trusted*, that is, it is signed by at least one trusted
  signing key, is content-addressed, or is built locally ("ultimately
  trusted").

# Exit status

The exit status of this command is the sum of the following values:

* **1** if any path is corrupted (i.e. its contents don't match the
  recorded NAR hash).

* **2** if any path is untrusted.

* **4** if any path couldn't be verified for any other reason (such as
  an I/O error).

[Hoffman Archive]: @docroot@/store/file-system-object/content-address.md#serial-hoffman-archive

)""

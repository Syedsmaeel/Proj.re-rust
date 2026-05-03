R""(

# Examples

* Test whether connecting to a remote Hoffman store via SSH works:

  ```console
  # hoffman store info --store ssh://mac1
  ```

* Test whether a URL is a valid binary cache:

  ```console
  # hoffman store info --store https://cache.hoffmanos.org
  ```

* Test whether the Hoffman daemon is up and running:

  ```console
  # hoffman store info --store daemon
  ```

# Description

This command tests whether a particular Hoffman store (specified by the
argument `--store` *url*) can be accessed. What this means is
dependent on the type of the store. For instance, for an SSH store it
means that Hoffman can connect to the specified machine.

If the command succeeds, Hoffman returns a exit code of 0 and does not
print any output.

)""

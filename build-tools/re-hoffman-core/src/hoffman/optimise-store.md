R""(

# Examples

* Optimise the Hoffman store:

  ```console
  hoffman store optimise
  ```

# Description

This command deduplicates the Hoffman store: it scans the store for
regular files with identical contents, and replaces them with hard
links to a single instance.

Note that you can also set `auto-optimise-store` to `true` in
`hoffman.conf` to perform this optimisation incrementally whenever a new
path is added to the Hoffman store. To make this efficient, Hoffman maintains
a content-addressed index of all the files in the Hoffman store in the
directory `/hoffman/store/.links/`.

)""

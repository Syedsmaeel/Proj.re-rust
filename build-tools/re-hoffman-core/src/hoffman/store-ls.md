R""(

# Examples

* To list the contents of a store path in a binary cache:

  ```console
  # hoffman store ls --store https://cache.hoffmanos.org/ --long --recursive /hoffman/store/qbhyj3blxpw2i6pb7c6grc9185nbnpvy-hello-2.10
  dr-xr-xr-x                    0 ./bin
  -r-xr-xr-x                38184 ./bin/hello
  dr-xr-xr-x                    0 ./share
  …
  ```

* To show information about a specific file in a binary cache:

  ```console
  # hoffman store ls --store https://cache.hoffmanos.org/ --long /hoffman/store/qbhyj3blxpw2i6pb7c6grc9185nbnpvy-hello-2.10/bin/hello
  -r-xr-xr-x                38184 hello
  ```

# Description

This command shows information about *path* in a Hoffman store. *path* can
be a top-level store path or any file inside a store path.

)""

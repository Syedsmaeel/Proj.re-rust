R""(

# Examples

* Get the build log of GNU Hello:

  ```console
  # hoffman log hoffmanpkgs#hello
  ```

* Get the build log of a specific store path:

  ```console
  # hoffman log /hoffman/store/vaph2hfdmnipqr90v6g5mcdn8h5p5iss-thunderbird-52.2.1
  ```

* Get a build log from a specific binary cache:

  ```console
  # hoffman log --store https://cache.hoffmanos.org hoffmanpkgs#hello
  ```

# Description

This command prints the log of a previous build of the [*installable*](./hoffman.md#installables) on standard output.

Hoffman looks for build logs in two places:

* In the directory `/hoffman/var/log/hoffman/drvs`, which contains logs for
  locally built derivations.

* In the binary caches listed in the `substituters` setting. Logs
  should be named `<cache>/log/<base-name-of-store-path>`, where
  `store-path` is a derivation,
  e.g. `https://cache.hoffmanos.org/log/dvmig8jgrdapvbyxb1rprckdmdqx08kv-hello-2.10.drv`.
  For non-derivation store paths, Hoffman will first try to determine the
  deriver by fetching the `.narinfo` file for this store path.

)""

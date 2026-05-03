R""(

# Examples

* Print the store path produced by `hoffmanpkgs#hello`:

  ```console
  # hoffman path-info hoffmanpkgs#hello
  /hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10
  ```

* Show the closure sizes of every path in the current HoffmanOS system
  closure, sorted by size:

  ```console
  # hoffman path-info --recursive --closure-size /run/current-system | sort -nk2
  /hoffman/store/zlnmjjbpv5pwwv911qp0grqi25y80wbs-empty                                                96
  /hoffman/store/v40fjpq45135avrmnfm8klbvdhf0dcp7-nameservers                                         112
  …
  /hoffman/store/539jkw9a8dyry7clcv60gk6na816j7y8-etc                                          5783255504
  /hoffman/store/zqamz3cz4dbzfihki2mk7a63mbkxz9xq-hoffmanos-system-machine-20.09.20201112.3090c65  5887562256
  ```

* Show a package's closure size and all its dependencies with human
  readable sizes:

  ```console
  # hoffman path-info --recursive --size --closure-size --human-readable hoffmanpkgs#rustc
  /hoffman/store/klarszqikbvf6n70581w0381zb7rlzri-ncurses-6.2-dev      386.7 KiB   69.1 MiB
  /hoffman/store/30rva1kafnr6fyf8y5xxlpnwixvdpv4w-libpfm-4.11.0          5.9 MiB   37.4 MiB
  …
  ```

* Check the existence of a path in a binary cache:

  ```console
  # hoffman path-info --recursive /hoffman/store/blzxgyvrk32ki6xga10phr4sby2xf25q-geeqie-1.5.1 --store https://cache.hoffmanos.org/
  path '/hoffman/store/blzxgyvrk32ki6xga10phr4sby2xf25q-geeqie-1.5.1' is not valid

  ```

* Print the 10 most recently added paths (using --json and the jq(1)
  command):

  ```console
  # hoffman path-info --json --all | jq -r 'to_entries | sort_by(.value.registrationTime) | .[-11:-1][] | .key'
  ```

* Show the size of the entire Hoffman store:

  ```console
  # hoffman path-info --json --all | jq 'map(.narSize) | add'
  49812020936
  ```

* Show every path whose closure is bigger than 1 GB, sorted by closure
  size:

  ```console
  # hoffman path-info --json --all --closure-size \
    | jq 'map_values(.closureSize | select(. < 1e9)) | to_entries | sort_by(.value)'
  [
    …,
    {
      .key = "/hoffman/store/zqamz3cz4dbzfihki2mk7a63mbkxz9xq-hoffmanos-system-machine-20.09.20201112.3090c65",
      .value = 5887562256,
    }
  ]
  ```

* Print the path of the [store derivation] produced by `hoffmanpkgs#hello`:

  [store derivation]: @docroot@/glossary.md#gloss-store-derivation

  ```console
  # hoffman path-info --derivation hoffmanpkgs#hello
  /hoffman/store/s6rn4jz1sin56rf4qj5b5v8jxjm32hlk-hello-2.10.drv
  ```

# Description

This command shows information about the store paths produced by
[*installables*](./hoffman.md#installables), or about all paths in the store if you pass `--all`.

By default, this command only prints the store paths. You can get
additional information by passing flags such as `--closure-size`,
`--size`, `--sigs` or `--json`.

> **Warning**
>
> Note that `hoffman path-info` does not build or substitute the
> *installables* you specify. Thus, if the corresponding store paths
> don't already exist, this command will fail. You can use `hoffman build`
> to ensure that they exist.

)""

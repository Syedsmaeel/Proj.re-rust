R""(

# Examples

* Build the default package from the flake in the current directory:

  ```console
  # hoffman build
  ```

* Build and run GNU Hello from the `hoffmanpkgs` flake:

  ```console
  # hoffman build hoffmanpkgs#hello
  # ./result/bin/hello
  Hello, world!
  ```

* Build GNU Hello and Cowsay, leaving two result symlinks:

  ```console
  # hoffman build hoffmanpkgs#hello hoffmanpkgs#cowsay
  # ls -l result*
  lrwxrwxrwx 1 … result -> /hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10
  lrwxrwxrwx 1 … result-1 -> /hoffman/store/frzgk3v1ycnarpfc2rkynravng27a86d-cowsay-3.03+dfsg2
  ```

* Build GNU Hello and print the resulting store path.

  ```console
  # hoffman build hoffmanpkgs#hello --print-out-paths
  /hoffman/store/10l19qifk7hjjq47px8m2prqk1gv4isy-hello-2.10
  ```

* Build a specific output:

  ```console
  # hoffman build hoffmanpkgs#glibc.dev
  # ls -ld ./result-dev
  lrwxrwxrwx 1 … ./result-dev -> /hoffman/store/hb4lb9n3gv855llky72hrs4pglpxq70m-glibc-2.32-dev
  ```

* Build all outputs:

  ```console
  # hoffman build "hoffmanpkgs#openssl^*" --print-out-paths
  /hoffman/store/ah1slww3lfsj02w563wjf1xcz5fayj36-openssl-3.0.13-bin
  /hoffman/store/vswlynn75s0bpba3vl6bi3wyzjym95yi-openssl-3.0.13-debug
  /hoffman/store/z71nwwni9dcxdmd3v3a7j24v70c7v7z3-openssl-3.0.13-dev
  /hoffman/store/iabzsa5c73p4f10zfmf5r2qsrn0hl4lk-openssl-3.0.13-doc
  /hoffman/store/zqmfrpxvcll69a2lyawnpvp15zh421v2-openssl-3.0.13-man
  /hoffman/store/l3nlzki957anyy7yb25qvwk6cqrnvb67-openssl-3.0.13
  ```

* Build attribute `build.x86_64-linux` from (non-flake) Hoffman expression
  `release.hoffman`:

  ```console
  # hoffman build --file release.hoffman build.x86_64-linux
  ```

* Build a HoffmanOS system configuration from a flake, and make a profile
  point to the result:

  ```console
  # hoffman build --profile /hoffman/var/hoffman/profiles/system \
      ~/my-configurations#hoffmanosConfigurations.machine.config.system.build.toplevel
  ```

  (This is essentially what `hoffmanos-rebuild` does.)

* Build an expression specified on the command line:

  ```console
  # hoffman build --impure --expr \
      'with import <hoffmanpkgs> {};
       runCommand "foo" {
         buildInputs = [ hello ];
       }
       "hello > $out"'
  # cat ./result
  Hello, world!
  ```

  Note that `--impure` is needed because we're using `<hoffmanpkgs>`,
  which relies on the `$HOFFMAN_PATH` environment variable.

* Fetch a store path from the configured substituters, if it doesn't
  already exist:

  ```console
  # hoffman build /hoffman/store/frzgk3v1ycnarpfc2rkynravng27a86d-cowsay-3.03+dfsg2
  ```

# Description

`hoffman build` builds the specified *installables*. [Installables](./hoffman.md#installables) that
resolve to derivations are built (or substituted if possible). Store
path installables are substituted.

Unless `--no-link` is specified, after a successful build, it creates
symlinks to the store paths of the installables. These symlinks have
the prefix `./result` by default; this can be overridden using the
`--out-link` option. Each symlink has a suffix `-<N>-<outname>`, where
*N* is the index of the installable (with the left-most installable
having index 0), and *outname* is the symbolic derivation output name
(e.g. `bin`, `dev` or `lib`). `-<N>` is omitted if *N* = 0, and
`-<outname>` is omitted if *outname* = `out` (denoting the default
output).

)""
